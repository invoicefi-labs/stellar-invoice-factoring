#![allow(dead_code)]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, String, Map, Symbol};

#[contracttype]
pub enum DataKey {
    Admin,
    UsdcToken,
    ProtocolFeeBps,
    NextInvoiceId,
    NextBidId,
    Invoice(u64),
    Bids(u64),
    InvoiceOwner(u64),
}

#[contracttype]
pub enum InvoiceStatus {
    Created,
    Listed,
    Factored,
    Settled,
    Cancelled,
}

#[contracttype]
pub struct Invoice {
    pub id: u64,
    pub business: Address,
    pub debtor: Address,
    pub amount: u128,
    pub due_date: u64,
    pub status: InvoiceStatus,
    pub metadata_hash: String,
    pub created_at: u64,
}

#[contracttype]
pub struct Bid {
    pub id: u64,
    pub invoice_id: u64,
    pub lender: Address,
    pub discount_rate_bps: u64, // basis points (e.g. 300 = 3%)
    pub advance_amount: u128,
    pub timestamp: u64,
}

#[contract]
pub struct InvoiceFactoring;

#[contractimpl]
impl InvoiceFactoring {
    pub fn initialize(env: Env, admin: Address, usdc: Address, protocol_fee_bps: u64) {
        env.storage().set(&DataKey::Admin, &admin);
        env.storage().set(&DataKey::UsdcToken, &usdc);
        env.storage().set(&DataKey::ProtocolFeeBps, &protocol_fee_bps);
        env.storage().set(&DataKey::NextInvoiceId, &1u64);
        env.storage().set(&DataKey::NextBidId, &1u64);
    }

    pub fn create_invoice(
        env: Env,
        debtor: Address,
        amount: u128,
        due_date: u64,
        metadata_hash: String,
    ) -> u64 {
        let caller = env.invoker();
        caller.require_auth();

        if amount == 0 {
            panic!("invoice amount must be > 0");
        }
        if due_date <= env.ledger().timestamp() {
            panic!("due date must be in the future");
        }

        let mut next_id: u64 = env.storage().get(&DataKey::NextInvoiceId).unwrap();
        let invoice = Invoice {
            id: next_id,
            business: caller.clone(),
            debtor,
            amount,
            due_date,
            status: InvoiceStatus::Created,
            metadata_hash,
            created_at: env.ledger().timestamp(),
        };

        env.storage().set(&DataKey::Invoice(next_id), &invoice);
        env.storage().set(&DataKey::InvoiceOwner(next_id), &caller);
        env.storage().set(&DataKey::Bids(next_id), &Vec::<Bid>::new(&env));
        env.storage().set(&DataKey::NextInvoiceId, &(next_id + 1));

        next_id
    }

    pub fn list_for_factoring(env: Env, invoice_id: u64, _min_advance_rate_bps: u64) {
        let caller = env.invoker();
        caller.require_auth();

        let mut invoice: Invoice = env.storage()
            .get(&DataKey::Invoice(invoice_id))
            .unwrap_or_else(|| panic!("invoice not found"));

        let owner: Address = env.storage().get(&DataKey::InvoiceOwner(invoice_id)).unwrap();
        if owner != caller {
            panic!("only invoice owner can list");
        }

        match invoice.status {
            InvoiceStatus::Created => {
                invoice.status = InvoiceStatus::Listed;
                env.storage().set(&DataKey::Invoice(invoice_id), &invoice);
            }
            _ => panic!("invoice must be in Created status to list"),
        }
    }

    pub fn submit_bid(
        env: Env,
        invoice_id: u64,
        discount_rate_bps: u64,
        advance_amount: u128,
    ) -> u64 {
        let caller = env.invoker();
        caller.require_auth();

        let invoice: Invoice = env.storage()
            .get(&DataKey::Invoice(invoice_id))
            .unwrap_or_else(|| panic!("invoice not found"));

        match invoice.status {
            InvoiceStatus::Listed => {}
            _ => panic!("invoice is not listed for factoring"),
        }

        if advance_amount >= invoice.amount {
            panic!("advance must be less than invoice amount");
        }

        let mut next_bid_id: u64 = env.storage().get(&DataKey::NextBidId).unwrap();
        let bid = Bid {
            id: next_bid_id,
            invoice_id,
            lender: caller,
            discount_rate_bps,
            advance_amount,
            timestamp: env.ledger().timestamp(),
        };

        let mut bids: Vec<Bid> = env.storage().get(&DataKey::Bids(invoice_id)).unwrap();
        bids.push_back(bid);
        env.storage().set(&DataKey::Bids(invoice_id), &bids);
        env.storage().set(&DataKey::NextBidId, &(next_bid_id + 1));

        next_bid_id
    }

    pub fn accept_bid(env: Env, invoice_id: u64, bid_id: u64) {
        let caller = env.invoker();
        caller.require_auth();

        let mut invoice: Invoice = env.storage()
            .get(&DataKey::Invoice(invoice_id))
            .unwrap_or_else(|| panic!("invoice not found"));

        let owner: Address = env.storage().get(&DataKey::InvoiceOwner(invoice_id)).unwrap();
        if owner != caller {
            panic!("only invoice owner can accept bids");
        }

        let bids: Vec<Bid> = env.storage().get(&DataKey::Bids(invoice_id)).unwrap();
        let mut accepted_bid: Option<Bid> = None;
        for b in bids.iter() {
            if b.id == bid_id {
                accepted_bid = Some(b);
                break;
            }
        }
        let bid = accepted_bid.unwrap_or_else(|| panic!("bid not found"));

        // Transfer invoice ownership to lender
        env.storage().set(&DataKey::InvoiceOwner(invoice_id), &bid.lender);
        invoice.status = InvoiceStatus::Factored;
        env.storage().set(&DataKey::Invoice(invoice_id), &invoice);

        // TODO: USDC transfer of advance_amount from lender to business
        // This would use the token contract interface
    }

    pub fn settle_invoice(env: Env, invoice_id: u64) {
        let caller = env.invoker();
        caller.require_auth();

        let mut invoice: Invoice = env.storage()
            .get(&DataKey::Invoice(invoice_id))
            .unwrap_or_else(|| panic!("invoice not found"));

        // Only debtor can settle
        if invoice.debtor != caller {
            panic!("only the debtor can settle");
        }

        match invoice.status {
            InvoiceStatus::Factored => {}
            _ => panic!("invoice must be factored to settle"),
        }

        let owner: Address = env.storage().get(&DataKey::InvoiceOwner(invoice_id)).unwrap();
        let protocol_fee_bps: u64 = env.storage().get(&DataKey::ProtocolFeeBps).unwrap();
        let fee = (invoice.amount * u128::from(protocol_fee_bps)) / 10000;
        let lender_payout = invoice.amount - fee;

        invoice.status = InvoiceStatus::Settled;
        env.storage().set(&DataKey::Invoice(invoice_id), &invoice);

        // TODO: USDC transfer of lender_payout to owner (lender)
        // TODO: USDC transfer of fee to admin
    }

    pub fn cancel_listing(env: Env, invoice_id: u64) {
        let caller = env.invoker();
        caller.require_auth();

        let mut invoice: Invoice = env.storage()
            .get(&DataKey::Invoice(invoice_id))
            .unwrap_or_else(|| panic!("invoice not found"));

        let owner: Address = env.storage().get(&DataKey::InvoiceOwner(invoice_id)).unwrap();
        if owner != caller {
            panic!("only invoice owner can cancel");
        }

        match invoice.status {
            InvoiceStatus::Listed => {
                invoice.status = InvoiceStatus::Cancelled;
                env.storage().set(&DataKey::Invoice(invoice_id), &invoice);
            }
            _ => panic!("only listed invoices can be cancelled"),
        }
    }

    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        env.storage()
            .get(&DataKey::Invoice(invoice_id))
            .unwrap_or_else(|| panic!("invoice not found"))
    }

    pub fn get_bids(env: Env, invoice_id: u64) -> Vec<Bid> {
        env.storage()
            .get(&DataKey::Bids(invoice_id))
            .unwrap_or(Vec::new(&env))
    }
}
