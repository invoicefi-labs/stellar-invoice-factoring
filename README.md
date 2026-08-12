
<!-- fix(#12): add admin pause/unpause controls -->
# 📄 Stellar Invoice Factoring Protocol

A decentralized invoice financing protocol on the Stellar network using Soroban smart contracts. SMEs tokenize unpaid invoices, and DeFi lenders compete to discount them — providing instant liquidity without traditional banking delays.

## Overview

Small businesses wait 30-90 days for invoice payments. Traditional invoice factoring is slow, expensive, and opaque. **Stellar Invoice Factoring Protocol** lets businesses:

1. **Tokenize** an unpaid invoice as an on-chain asset
2. **List** it for discounting at a desired advance rate
3. Receive **instant liquidity** from competing lenders
4. **Settle** automatically when the debtor pays

## Why

- **Instant liquidity** — no 30-day waits, no bank approvals
- **Competitive pricing** — lenders bid against each other, lowering discount rates
- **Transparent** — every invoice, bid, and settlement is on-chain
- **Global access** — any lender worldwide can participate via Stellar
- **Low fees** — Soroban's sub-cent transactions make small invoices viable

## Architecture

### Soroban Contract (`contracts/invoice-factoring`)
| Function | Description |
|---|---|
| `initialize` | Set admin, USDC token address |
| `create_invoice` | Business tokenizes an invoice (debtor, amount, due_date, metadata) |
| `list_for_factoring` | Business lists invoice for discounting with desired advance rate |
| `submit_bid` | Lender bids with discount rate and advance amount |
| `accept_bid` | Business accepts a bid — advance transferred to business, invoice NFT to lender |
| `settle_invoice` | Debtor pays full amount — lender receives payment, protocol fee deducted |
| `cancel_listing` | Business cancels an open factoring listing |
| `get_invoice` | View invoice details and status |
| `get_bids` | View all bids for a listed invoice |

### Frontend (`frontend`)
React + Vite + Freighter wallet integration for:
- Businesses: create invoices, list for factoring, review bids, accept offers
- Lenders: browse factoring opportunities, submit bids, manage portfolio
- Debtors: view and settle invoices

## Flows

1. **Business** calls `create_invoice(debtor, amount, due_date)` — invoice tokenized on-chain
2. **Business** calls `list_for_factoring(invoice_id, min_advance_rate)` — listing created
3. **Lender** calls `submit_bid(invoice_id, discount_rate, advance_amount)` — bid placed
4. **Business** calls `accept_bid(invoice_id, bid_id)` — advance USDC sent to business, invoice ownership transferred to lender
5. **Debtor** calls `settle_invoice(invoice_id)` — full amount paid, lender receives payment minus protocol fee

## Build & Test

```bash
cd contracts/invoice-factoring && cargo test
cd ../../frontend && npm install && npm run dev
```

## License

MIT
