use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use rust_decimal::prelude::*;
use tracing::{error, info, instrument, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct Accounting {
    #[builder(default = String::from("accounting"))]
    identifier: String,
    time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    #[builder(default = Ledger::new())]
    ledger: Arc<Ledger>,
}

impl Accounting {
    async fn venue_order_fill(&self, _order: &VenueOrder) {
        info!(target: "accounting", "received fill");

        // self.ledger.
    }

    /// Get debit and credit assets
    /// For buy orders, the credit asset is the base asset and the debit asset is the quote asset.
    /// For sell orders, the credit asset is the quote asset and the debit asset is the base asset.
    pub async fn spot_fill(&self, order: &VenueOrder) {
        let transfer_group_id = Uuid::new_v4();
        let event_time = self.time.now().await;

        let venue = order.instrument.venue.clone();
        let (credit_asset, debit_asset) = if order.side == MarketSide::Buy {
            (
                Tradable::Asset(order.instrument.base_asset.clone()),
                Tradable::Asset(order.instrument.quote_asset.clone()),
            )
        } else {
            (
                Tradable::Asset(order.instrument.quote_asset.clone()),
                Tradable::Asset(order.instrument.base_asset.clone()),
            )
        };

        let (credit_amount, debit_amount) = if order.side == MarketSide::Buy {
            (order.quantity, order.quantity * order.price)
        } else {
            (order.quantity * order.price, order.quantity)
        };

        // Debit transfer
        let debit_account =
            self.ledger
                .find_or_create_account(&venue, &debit_asset, &AccountOwner::User, &AccountType::Spot);
        let credit_account =
            self.ledger
                .find_or_create_account(&venue, &debit_asset, &AccountOwner::Venue, &AccountType::Spot);

        let t1 = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(debit_asset)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(debit_amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        // Credit transfer
        let debit_account =
            self.ledger
                .find_or_create_account(&venue, &credit_asset, &AccountOwner::Venue, &AccountType::Spot);
        let credit_account =
            self.ledger
                .find_or_create_account(&venue, &credit_asset, &AccountOwner::User, &AccountType::Spot);

        let t2 = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(credit_asset)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(credit_amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        let res = self.ledger.apply_transfers(&[t1, t2]).await;
        match res {
            Ok(_) => {
                info!(target: "accounting", "Transfers applied successfully");
            }
            Err(e) => {
                error!(target: "accounting", "Failed to apply transfers: {}", e);
            }
        }
    }

    // pub async fn margin_fill(&self, order: &VenueOrder) {
    //     // info!(target: "accounting", "Starting Margin Trade...");
    //     // info!(target: "accounting", "Side: {}", side);
    //     // info!(target: "accounting", "Price: {}", price);
    //     // info!(target: "accounting", "Amount: {}", amount);
    //     // info!(target: "accounting", "Margin Rate: {}", margin_rate);
    //     // info!(target: "accounting", "Commission Rate: {}", commission_rate);
    //     let event_time = self.time.now().await;
    //     let instrument = order.instrument.clone();
    //     let strategy = order.strategy.clone();
    //     let venue = order.instrument.venue.clone();
    //     let price = order.last_fill_price;
    //     let amount = order.last_fill_quantity;
    //     let side = order.side;
    //     let margin_rate = dec!(0.05);
    //     let inst_asset = Tradable::Instrument(order.instrument.clone());
    //     let margin_asset = Tradable::Asset(order.instrument.margin_asset.clone());
    //     let commission_asset =
    //         Tradable::Asset(order.commission_asset.unwrap_or_else(|| order.instrument.margin_asset.clone()));

    //     //  Find or create necessary accounts
    //     let user_margin =
    //         self.ledger
    //             .find_or_create_account(&venue, &margin_asset, &AccountOwner::User, &AccountType::Margin);
    //     let venue_margin =
    //         self.ledger
    //             .find_or_create_account(&venue, &margin_asset, &AccountOwner::Venue, &AccountType::Margin);
    //     let user_inst =
    //         self.ledger
    //             .find_or_create_account(&venue, &inst_asset, &AccountOwner::User, &AccountType::Instrument);
    //     let venue_inst =
    //         self.ledger
    //             .find_or_create_account(&venue, &inst_asset, &AccountOwner::Venue, &AccountType::Instrument);
    //     let venue_spot =
    //         self.ledger
    //             .find_or_create_account(&venue, &commission_asset, &AccountOwner::Venue, &AccountType::Spot);

    //     let (cost_basis, current_position) =
    //         self.ledger.current_position(&order.strategy.unwrap(), Some(&instrument)).await;
    //     info!(target: "accounting", "Cost Basis: {}, Current Position {}", cost_basis, current_position);
    //     let new_position = match side {
    //         MarketSide::Buy => current_position + amount,
    //         MarketSide::Sell => current_position - amount,
    //     };
    //     info!(target: "accounting", "New Position after {} will be: {}", side, new_position);

    //     // Calculate amount closed and PnL
    //     let amount_closed = if (current_position > Decimal::ZERO && new_position <= Decimal::ZERO)
    //         || (current_position < Decimal::ZERO && new_position >= Decimal::ZERO)
    //     {
    //         info!(target: "accounting", "Position will fully close: {} -> {}", current_position, new_position);
    //         current_position.abs() // Full close before flip
    //     } else {
    //         info!(target: "accounting", "Position will not close fully: {} -> {}", current_position, new_position);
    //         amount.min(current_position.abs()) // Partial close
    //     };
    //     info!(target: "accounting", "Amount closed: {}", amount_closed);

    //     let entry_price = if !current_position.is_zero() {
    //         cost_basis / current_position.abs()
    //     } else {
    //         Decimal::ZERO
    //     };
    //     info!(target: "accounting", "Entry price from ledger: {}", entry_price);
    //     let pnl = if current_position > Decimal::ZERO {
    //         info!(target: "accounting", "Calculating PnL for long position");
    //         (price - entry_price) * amount_closed
    //     } else if current_position < Decimal::ZERO {
    //         info!(target: "accounting", "Calculating PnL for short position");
    //         (entry_price - price) * amount_closed
    //     } else {
    //         info!(target: "accounting", "No PnL for flat position");
    //         dec!(0)
    //     };

    //     // Margin adjustments
    //     let margin_delta = if amount_closed > Decimal::ZERO && current_position.signum() == new_position.signum() {
    //         let current_margin = self.ledger.margin_posted(&strategy, Some(&instrument)).await;
    //         let closing_margin = current_margin * (amount_closed / current_position.abs());
    //         -closing_margin
    //     } else if amount_closed.is_zero()
    //         && (current_position.signum() == new_position.signum() || current_position.is_zero())
    //     {
    //         let posting = new_position.abs() * price * margin_rate;
    //         posting
    //     } else {
    //         let posting = new_position.abs() * price * margin_rate;
    //         let current_margin = self.ledger.margin_posted(&strategy, Some(&instrument)).await;
    //         let closing_margin = current_margin * (amount_closed / current_position.abs());
    //         posting - closing_margin
    //     };
    //     info!(target: "accounting", "Margin delta: {}", margin_delta);

    //     //  Calculate commission
    //     let commission = amount * price * commission_rate;
    //     info!(target: "accounting", "Commission: {}", commission);

    //     // Step 7: Create transfers
    //     let transfer_group_id = Uuid::new_v4();
    //     let mut transfers = Vec::new();

    //     // Margin adjustment
    //     if margin_delta > dec!(0) {
    //         // Post additional margin
    //         transfers.push(Arc::new(
    //             Transfer::builder()
    //                 .event_time(event_time)
    //                 .transfer_group_id(transfer_group_id)
    //                 .asset(user_margin.asset.clone())
    //                 .strategy(Some(strategy.clone()))
    //                 .instrument(Some(instrument.clone()))
    //                 .debit_account(user_margin.clone())
    //                 .credit_account(venue_margin.clone())
    //                 .amount(margin_delta)
    //                 .unit_price(Decimal::ONE)
    //                 .transfer_type(TransferType::Margin)
    //                 .build(),
    //         ));
    //     } else if margin_delta < dec!(0) {
    //         // Free margin
    //         transfers.push(Arc::new(
    //             Transfer::builder()
    //                 .event_time(event_time)
    //                 .transfer_group_id(transfer_group_id)
    //                 .asset(venue_margin.asset.clone())
    //                 .strategy(Some(strategy.clone()))
    //                 .instrument(Some(instrument.clone()))
    //                 .debit_account(venue_margin.clone())
    //                 .credit_account(user_margin.clone())
    //                 .amount(margin_delta.abs())
    //                 .unit_price(Decimal::ONE)
    //                 .transfer_type(TransferType::Margin)
    //                 .build(),
    //         ));
    //     }

    //     // Commission payment
    //     transfers.push(Arc::new(
    //         Transfer::builder()
    //             .event_time(event_time)
    //             .transfer_group_id(transfer_group_id)
    //             .asset(user_margin.asset.clone())
    //             .strategy(Some(strategy.clone()))
    //             .instrument(Some(instrument.clone()))
    //             .debit_account(user_margin.clone())
    //             .credit_account(venue_spot.clone())
    //             .amount(commission)
    //             .unit_price(Decimal::ONE)
    //             .transfer_type(TransferType::Commission)
    //             .build(),
    //     ));

    //     // Instrument trade
    //     let (debit_inst, credit_inst) = if side == MarketSide::Buy {
    //         (venue_inst.clone(), user_inst.clone())
    //     } else {
    //         (user_inst.clone(), venue_inst.clone())
    //     };
    //     transfers.push(Arc::new(
    //         Transfer::builder()
    //             .event_time(event_time)
    //             .transfer_group_id(transfer_group_id)
    //             .asset(debit_inst.asset.clone())
    //             .strategy(Some(strategy.clone()))
    //             .instrument(Some(instrument.clone()))
    //             .debit_account(debit_inst)
    //             .credit_account(credit_inst)
    //             .amount(amount)
    //             .unit_price(price)
    //             .transfer_type(TransferType::Trade)
    //             .build(),
    //     ));

    //     // PnL transfer
    //     if amount_closed > dec!(0) {
    //         if pnl > Decimal::ZERO {
    //             // Profit: venue_spot -> user_margin
    //             transfers.push(Arc::new(
    //                 Transfer::builder()
    //                     .event_time(event_time)
    //                     .transfer_group_id(transfer_group_id)
    //                     .asset(venue_spot.asset.clone())
    //                     .strategy(Some(strategy.clone()))
    //                     .instrument(Some(instrument.clone()))
    //                     .debit_account(venue_spot.clone())
    //                     .credit_account(user_margin.clone())
    //                     .amount(pnl)
    //                     .unit_price(Decimal::ONE)
    //                     .transfer_type(TransferType::Pnl)
    //                     .build(),
    //             ));
    //         } else if pnl < dec!(0) {
    //             // Loss: user_margin -> venue_spot
    //             transfers.push(Arc::new(
    //                 Transfer::builder()
    //                     .event_time(event_time)
    //                     .transfer_group_id(transfer_group_id)
    //                     .asset(user_margin.asset.clone())
    //                     .strategy(Some(strategy.clone()))
    //                     .instrument(Some(instrument.clone()))
    //                     .debit_account(user_margin.clone())
    //                     .credit_account(venue_spot.clone())
    //                     .amount(pnl.abs())
    //                     .unit_price(Decimal::ONE)
    //                     .transfer_type(TransferType::Pnl)
    //                     .build(),
    //             ));
    //         }
    //     }

    //     // Apply transfers
    //     let res = self.ledger.apply_transfers(&transfers).await;
    //     match res {
    //         Ok(transfers) => {
    //             info!(target: "accounting", "Transfers applied successfully");
    //         }
    //         Err(e) => {
    //             error!(target: "accounting", "Failed to apply transfers: {}", e);
    //         }
    //     }
    //     // self.publisher.publish(Event::TransferNew(transfers)).await;
    // }
}

#[async_trait]
impl Runnable for Accounting {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            e => warn!(target: "accounting", "received unused event {}", e),
        }
    }
}
