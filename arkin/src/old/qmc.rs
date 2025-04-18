#![allow(non_snake_case)]
use clarabel::{algebra::*, solver::*};
use metromc::{
    gaussian::Gaussian,
    sampler::{Config, Sampler},
    Pdf,
};
use statrs::{distribution::Continuous, statistics::Statistics};
use tinyrand::{Rand, Seeded, Wyrand};
use typed_builder::TypedBuilder;

pub struct LogNormal {
    dist: statrs::distribution::LogNormal,
}
impl LogNormal {
    #[inline(always)]
    pub fn new(mean: f64, std_dev: f64) -> Self {
        Self {
            dist: statrs::distribution::LogNormal::new(mean, std_dev).unwrap(),
        }
    }
}

impl Pdf for LogNormal {
    #[inline(always)]
    fn prob(&self, x: f64) -> f64 {
        self.dist.pdf(x)
    }
}

const TIME_STEPS: f64 = 60.;
const RISK_FREE_RATE: f64 = 0.00;
const NUM_SCENARIOS: usize = 10000;

const LAMBDA: f64 = 0.0001;
const MAX_EXPOSURE_LONG: f64 = 0.8;
const MAX_EXPOSURE_SHORT: f64 = 0.8;
const MAX_LONG_PER_ASSET: f64 = 0.3;
const MAX_SHORT_PER_ASSET: f64 = 0.3;

fn main() {
    // Initialize data
    let u = vec![
        inst_ret_1(),
        inst_ret_2(),
        inst_ret_3(),
        inst_ret_1().iter().map(|r| r * -1.).collect(),
        inst_ret_2().iter().map(|r| r * -1.).collect(),
        inst_ret_3().iter().map(|r| r * -1.).collect(),
    ];

    // Expected Returns
    let mut mu = u.iter().map(|u| u.mean()).collect::<Vec<_>>(); // expected returns
    mu.iter_mut().for_each(|r| *r = *r * TIME_STEPS);
    println!("Expected Returns: {:?}", mu);

    // Volatility (Standard Deviation)
    let mut sigma = [inst_ret_1().std_dev(), inst_ret_2().std_dev(), inst_ret_3().std_dev()]; // standard deviations
    sigma.iter_mut().for_each(|s| *s = *s);

    // Step 1: Generate scenarios
    let scenarios = generate_scenarios(&mu, &sigma);

    // Step 3: Calculate the optimal weights
    let mvo = MeanVarianceOptimization::builder()
        .lambda(LAMBDA)
        .max_exposure_long(MAX_EXPOSURE_LONG)
        .max_exposure_short(MAX_EXPOSURE_SHORT)
        .max_long_per_asset(MAX_LONG_PER_ASSET)
        .max_short_per_asset(MAX_SHORT_PER_ASSET)
        .build();
    let weights = mvo.solve(&u, &mu);

    // Step 3: Let's see what a equal-weighted portfolio looks like
    let mut metrics = Vec::with_capacity(NUM_SCENARIOS);
    for scenario in scenarios.iter() {
        let mu = compute_portfolio_return(&weights, scenario);
        let v = compute_portfolio_variance(&weights, &compute_covariance_matrix(&u));
        let s = compute_sharpe_ratio(mu, v);
        metrics.push((mu, v, s));
    }

    // Step 4: Order by Sharpe ratio
    metrics.sort_unstable_by(|(_mu1, _v1, s1), (_m2, _v2, s2)| s2.partial_cmp(&s1).unwrap());
    // Calculate the average with statrs
    let avg_return = metrics.iter().map(|(r, _s, _sharpe)| *r).mean();
    let avg_risk = metrics.iter().map(|(_r, s, _sharpe)| *s).mean();
    let avg_sharpe = metrics.iter().map(|(_r, _s, sharpe)| *sharpe).mean();
    println!(
        "Average Return: {:.8}, Average Risk: {:.8}, Average Sharpe: {:.8}",
        avg_return, avg_risk, avg_sharpe
    );

    // Step 5: Print the top 10 scenarios
    println!("Top 5 Scenarios:");
    for (i, (r, s, sharpe)) in metrics.iter().take(5).enumerate() {
        println!("Scenario {}: Return: {:.8}, Risk: {:.8}, Sharpe: {:.8}", i, r, s, sharpe);
    }
    println!("worst 5 Scenarios:");
    for (i, (r, s, sharpe)) in metrics.iter().rev().take(5).enumerate() {
        println!("Scenario {}: Return: {:.8}, Risk: {:.8}, Sharpe: {:.8}", i, r, s, sharpe);
    }
}

#[derive(Debug, TypedBuilder)]
pub struct MeanVarianceOptimization {
    lambda: f64,
    max_exposure_long: f64,
    max_exposure_short: f64,
    max_long_per_asset: f64,
    max_short_per_asset: f64,
}

impl MeanVarianceOptimization {
    pub fn solve(&self, u: &[Vec<f64>], mu: &[f64]) -> Vec<f64> {
        let lambda = self.lambda;

        let n = mu.len();

        // Assemble the covariance matrix
        let covariance_matrix = compute_covariance_matrix(u);
        // for row in &covariance_matrix {
        //     for val in row {
        //         print!("{:.5} ", val);
        //     }
        //     println!();
        // }

        // Problem definition
        let P_data = covariance_matrix
            .iter()
            .map(|row| row.iter().map(|val| 2.0 * val).collect())
            .collect::<Vec<Vec<f64>>>();
        let P = CscMatrix::from(&P_data);

        // Linear coefficients
        // let q = vec![0., 0., 0., 0., 0.];
        let q: Vec<f64> = mu.iter().map(|r| -lambda * r).collect();
        // Minimize the portfolio variance
        // q.iter_mut().enumerate().for_each(|(i, w)| *w += -gamma * forecast_var[i]);
        // q.iter_mut().enumerate().for_each(|(i, w)| *w -= tau * prev_weights[i]);

        // Constraints
        let (A, b, cones) = self.create_constraints(n);

        // Solve the problem
        let mut settings = DefaultSettings::default();
        settings.verbose = false;

        let mut solver = DefaultSolver::new(&P, &q, &A, &b, &cones, settings);
        solver.solve();

        let weights = solver.solution.x;
        for (i, w) in weights.iter().enumerate() {
            println!("Asset {}: {}%", i, (w * 100.).round());
        }
        println!("Sum of weights: {:.5}", weights.iter().sum::<f64>());
        println!(
            "Solution Status: {:?} with {:?} iterations",
            solver.solution.status, solver.solution.iterations
        );

        let expected_return = compute_portfolio_return(&weights, mu);
        let portfolio_variance = compute_portfolio_variance(&weights, &covariance_matrix);
        let risk = portfolio_variance.sqrt();
        let sharpe = compute_sharpe_ratio(expected_return, portfolio_variance);
        println!("Expected Return: {:.5}", expected_return);
        println!("Portfolio Variance: {:.5}", portfolio_variance);
        println!("Risk: {:.5}", risk);
        println!("Sharpe Ratio: {:.5}", sharpe);
        weights
    }

    fn create_constraints(&self, n: usize) -> (CscMatrix<f64>, Vec<f64>, [SupportedConeT<f64>; 1]) {
        let m = 2 + 2 * n;
        let max_exposure_long = self.max_exposure_long;
        let max_exposure_short = self.max_exposure_short;
        let max_long_per_asset = self.max_long_per_asset;
        let max_short_per_asset = self.max_short_per_asset;

        let mut I = Vec::with_capacity(m);
        let mut J = Vec::with_capacity(n);
        let mut V = Vec::new();

        // Max allocation constraint
        for i in 0..n {
            let column = i;
            let row = 0;
            let val = 1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        for i in 0..n {
            let column = i;
            let row = 1;
            let val = -1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        // Max long constraint
        for i in 0..n {
            let column = i;
            let row = i + 2;
            let val = 1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        // Max short constraint
        for i in 0..n {
            let column = i;
            let row = i + n + 2;
            let val = -1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        let A = CscMatrix::new_from_triplets(m, n, I, J, V);

        let mut b = vec![max_exposure_long, max_exposure_short];
        b.extend(vec![max_long_per_asset; n]);
        b.extend(vec![max_short_per_asset; n]);

        let cones = [NonnegativeConeT(m)];

        (A, b, cones)
    }
}

// Our Monte Carlo Simulation
fn generate_scenarios(mu: &[f64], sigma: &[f64]) -> Vec<Vec<f64>> {
    let mut scenarios = Vec::with_capacity(NUM_SCENARIOS);
    let mut rng = Wyrand::default();
    for _ in 0..NUM_SCENARIOS {
        let mut scenario = Vec::new();
        for (expected_return, std_dev) in mu.iter().zip(sigma) {
            let mu_sampler = Sampler::new(Config {
                rand: Wyrand::seed(rng.next_u64()),
                dist: Gaussian::new(*expected_return, *std_dev),
                range: -1.0..=1.0,
            });
            let prob_mu = mu_sampler.skip(1000).next().expect("mu_sampler failed");
            scenario.push(prob_mu);
        }
        scenarios.push(scenario);
    }
    scenarios
}

/// Function to compute the covariance matrix from data
fn compute_covariance_matrix(u: &[Vec<f64>]) -> Vec<Vec<f64>> {
    u.iter()
        .enumerate()
        .map(|(i, _)| {
            u.iter()
                .enumerate()
                .map(|(j, _)| u[i].clone().covariance(u[j].clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>()
}

fn compute_portfolio_return(weights: &[f64], expected_return: &[f64]) -> f64 {
    weights.iter().zip(expected_return.iter()).map(|(w, r)| w * r).sum()
}

/// Function to compute portfolio variance: w^T * Sigma * w
fn compute_portfolio_variance(weights: &[f64], covariance_matrix: &Vec<Vec<f64>>) -> f64 {
    let mut variance = 0.0;
    let n = weights.len();
    for i in 0..n {
        for j in 0..n {
            variance += weights[i] * covariance_matrix[i][j] * weights[j];
        }
    }
    variance
}

fn compute_sharpe_ratio(expected_return: f64, portfolio_variance: f64) -> f64 {
    (expected_return - RISK_FREE_RATE) / portfolio_variance.sqrt()
}

fn inst_ret_1() -> Vec<f64> {
    vec![
        0.0000153797991660012857013631,
        -0.0001222277625474973749382990,
        -0.0000141414668823517079132259,
        0.0004200659473339380444359725,
        0.0004459178159552625620340088,
        0.0001254244290119028520463140,
        -0.0000646003165830488934386790,
        0.0001633921172641759969795139,
        -0.0000227337753176365938359050,
        0.0000319681352468619248610783,
        0.0001390513135951420553996724,
        0.0004436062691537556091939553,
        -0.0000008283636117987284845427,
        -0.0003926516149261116775332029,
        0.0000218659298154676206972521,
        0.0000421815296481955903528063,
        0.0004097495059329089231350642,
        0.0000504755142198373510576142,
        0.0010768363536443583207761499,
        0.0012083370662564071981180373,
        0.0001926478673468783629526226,
        -0.0002644063806854255843632865,
        0.0008086661843055578887775696,
        0.0005325844577098509216793002,
        0.0002741218727393846638198895,
        -0.0005114923469415213217808126,
        -0.0002728137289069702556237102,
        0.0006689605217836459465636231,
        -0.0004941445826033992998990215,
        0.0006411538648528065259743103,
        0.0005739651706594616079638855,
        0.0009504529510155687824018373,
        0.0011729014660215158942601563,
        0.0002922167899852360330597873,
        0.0000573329424116374443830768,
        -0.0003273108361634677902216862,
        -0.0012572648471223375345494152,
        -0.0010938098532078383650232024,
        -0.0003968582795536375043533258,
        -0.0002184210139731048736459097,
        -0.0005913735999827364487935904,
        -0.0003593955488843981261255755,
        0.0003025056010727330661782365,
        -0.0006194840881204274268817760,
        -0.0000167648501549478644719447,
        0.0000797510165838474782922340,
        0.0000519167329346501649433799,
        0.0005521418851066953891434168,
        0.0006777226627629624770687134,
        0.0003497691887767937089346998,
        -0.0005829815877421389170212050,
        0.0004706449344680221548893476,
        0.0008603922995224714789457822,
        -0.0001752183686655202086737829,
        -0.0002637492625358577021181875,
        -0.0002251715823848746281841913,
        -0.0002212354337944162183800008,
        0.0001573450306431436104074983,
        0.0002971731751970069343260809,
        0.0000005006070111890143997302,
        0.0001903128416026226957480431,
    ]
}

fn inst_ret_2() -> Vec<f64> {
    vec![
        0.0001920247206166408310105222,
        -0.0002796697498751007843773264,
        0.0000774184340879940423860121,
        -0.0001210262920983981921472224,
        0.0006823900502280562739824743,
        0.0002476077103811020345119511,
        -0.0005998472476948304493816527,
        -0.0003570081705341163516424118,
        0.0000308584753702817012780555,
        -0.0002826172595327731259471324,
        0.0001278310498770266037630372,
        0.0005320810377114929877389825,
        0.0001901205676862433699089831,
        -0.0002034084099293645867227138,
        -0.0001853033638125359977634132,
        0.0005090528413409279041626254,
        0.0003264111151401160589379269,
        -0.0001560660133239014807655169,
        0.0004551748199541565074107715,
        0.0012153155022370413349786784,
        0.0000815762607723686690483975,
        -0.0004756124000791360843702093,
        0.0009411869405116030231594713,
        0.0003963521921082173774018258,
        0.0002186611059990835413807864,
        -0.0012027310371168271725687872,
        -0.0004272466836787676608678607,
        0.0009228395432534003523307045,
        -0.0005136920851495314842280073,
        -0.0001314608232994116945558631,
        0.0004925562305352961210703508,
        0.0013030430899574042850700236,
        0.0011650490790181133316087295,
        -0.0001769287685393598635238531,
        -0.0001863133638890919146940436,
        -0.0003617022512639825615548858,
        -0.0015560499148714629056770808,
        -0.0004003356826267491670724416,
        -0.0002803320012420448881607443,
        -0.0005616431702770287045445305,
        -0.0001531060657821595097528490,
        -0.0000002870740930320740460346,
        0.0005641523254794001071777588,
        -0.0002171420546599741703031380,
        0.0004024732193486846803461311,
        -0.0000136155266740028673594282,
        -0.0001372828928753726541728169,
        0.0001236252915716854074805170,
        0.0007001783931798271381848259,
        0.0003481495411832875154727152,
        -0.0007052460169852306193998750,
        0.0006188857995067574917895648,
        0.0005147201504649295263695211,
        0.0001481011837835867756358641,
        -0.0002517689328945298864748236,
        -0.0002388807475577313387696911,
        -0.0001878563290623461173091123,
        -0.0000170844582954167348112675,
        0.0000273067906489299017271759,
        -0.0001395092023152093099905624,
        0.0000617948378675202514968362,
    ]
}

fn inst_ret_3() -> Vec<f64> {
    vec![
        0.0003431852188994605767425379,
        -0.0001249888766173671631567147,
        0.0003077347379340679163011656,
        0.0002581414593343163662591047,
        0.0005632821822586077481402793,
        0.0005538345494340569712675162,
        -0.0003185259727533070305271354,
        -0.0006997380059060513302127859,
        0.0004309363710365868126717330,
        0.0003062394121550251056032762,
        0.0002228984908995424986751502,
        0.0000343910254786869093724099,
        0.0004880690489117320539149099,
        -0.0001886907393063033547513683,
        -0.0002262002654844115548052776,
        0.0008487990661153732973875085,
        -0.0001977036052664999632532337,
        -0.0002146345906924133351740035,
        0.0010035879039011740485177106,
        0.0015625819548909534052402977,
        0.0001652236445858581882683352,
        -0.0010676511393815771051249056,
        0.0009997239145547177037971714,
        0.0003846653439976874669696316,
        0.0006910713498874887986989317,
        -0.0002999084545050011860030836,
        0.0005433044796141819822541442,
        0.0002506680501994473771078354,
        0.0000665820732585754442554340,
        -0.0000672897593651680169503973,
        0.0009737529227082825302133786,
        0.0009535899317784650534832799,
        0.0018082679648501654049176267,
        -0.0001597615552946420966736337,
        -0.0004064278938119191742249004,
        -0.0006244704872395343936442107,
        -0.0013412328261000680430334787,
        -0.0006760991532654827905830342,
        -0.0002888171431067718637104194,
        -0.0007371471775744751737211487,
        0.0001896915266689868618861793,
        0.0002055492305723264348587555,
        0.0006287797603806162909063750,
        -0.0002238628632450561278610061,
        0.0007452186766654669643760882,
        0.0002381216829829866262317730,
        -0.0000586770047314570795423093,
        -0.0001880395305485667249415954,
        0.0002036635181052091356797709,
        0.0004898112407217495275267650,
        -0.0004989222046309250253037337,
        0.0004196852391216809505794189,
        0.0011062742741973525912940344,
        -0.0000825702375337770588067065,
        -0.0006679704022612766816723880,
        -0.0005963114550512475771584651,
        -0.0006513252102839031517638532,
        -0.0002429396217073272060233143,
        -0.0005196419269362663644904164,
        0.0006194331040450114160600032,
        -0.0002853464434088562926472806,
    ]
}
