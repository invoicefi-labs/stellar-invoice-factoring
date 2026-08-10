// fix(#11): rounding error in accrued interest
import { useState } from 'react';

type Invoice = {
  id: number;
  business: string;
  debtor: string;
  amount: number;
  dueDate: string;
  status: string;
};

type Bid = {
  id: number;
  lender: string;
  discountRate: number;
  advanceAmount: number;
};

export default function App() {
  const [tab, setTab] = useState<'create' | 'browse' | 'bids'>('browse');
  const [invoices, setInvoices] = useState<Invoice[]>([
    { id: 1, business: 'GABC...', debtor: 'GDEF...', amount: 15000, dueDate: '2024-12-01', status: 'Listed' },
  ]);
  const [bids, setBids] = useState<Bid[]>([
    { id: 1, lender: 'GXYZ...', discountRate: 3.5, advanceAmount: 13500 },
  ]);

  return (
    <div className="min-h-screen bg-blue-50 p-8">
      <header className="mb-8">
        <h1 className="text-3xl font-bold text-blue-800">📄 Stellar Invoice Factoring</h1>
        <p className="text-blue-600 mt-2">Turn unpaid invoices into instant liquidity on Stellar</p>
      </header>

      <nav className="flex gap-4 mb-6">
        {(['browse', 'create', 'bids'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 rounded-lg capitalize ${tab === t ? 'bg-blue-600 text-white' : 'bg-white text-blue-700'}`}
          >
            {t === 'browse' ? 'Browse Invoices' : t === 'create' ? 'Create Invoice' : 'View Bids'}
          </button>
        ))}
      </nav>

      {tab === 'browse' && (
        <div className="space-y-4">
          {invoices.map((inv) => (
            <div key={inv.id} className="bg-white rounded-xl shadow p-4 flex justify-between items-center">
              <div>
                <span className="font-semibold">Invoice #{inv.id}</span>
                <span className={`ml-3 px-2 py-1 rounded text-xs ${inv.status === 'Listed' ? 'bg-green-100 text-green-700' : 'bg-gray-100'}`}>
                  {inv.status}
                </span>
                <p className="text-sm text-gray-500 mt-1">Debtor: {inv.debtor.slice(0, 8)}... · Due: {inv.dueDate}</p>
              </div>
              <div className="text-right">
                <p className="text-2xl font-bold text-blue-700">{inv.amount.toLocaleString()} USDC</p>
              </div>
            </div>
          ))}
        </div>
      )}

      {tab === 'create' && (
        <div className="bg-white rounded-xl shadow p-6 max-w-lg">
          <h3 className="font-semibold text-lg mb-4">Create New Invoice</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-600 mb-1">Debtor Address</label>
              <input className="w-full border rounded-lg p-2" placeholder="G..." />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">Amount (USDC)</label>
              <input type="number" className="w-full border rounded-lg p-2" placeholder="15000" />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">Due Date</label>
              <input type="date" className="w-full border rounded-lg p-2" />
            </div>
            <button className="w-full bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700">
              Tokenize Invoice
            </button>
          </div>
        </div>
      )}

      {tab === 'bids' && (
        <div className="space-y-4">
          {bids.map((bid) => (
            <div key={bid.id} className="bg-white rounded-xl shadow p-4 flex justify-between items-center">
              <div>
                <span className="font-semibold">Bid #{bid.id}</span>
                <p className="text-sm text-gray-500 mt-1">Lender: {bid.lender.slice(0, 8)}...</p>
              </div>
              <div className="text-right">
                <p className="text-lg font-semibold text-blue-700">{bid.advanceAmount.toLocaleString()} USDC advance</p>
                <p className="text-sm text-gray-500">{bid.discountRate}% discount</p>
                <button className="mt-2 bg-green-600 text-white px-4 py-1 rounded-lg text-sm hover:bg-green-700">
                  Accept
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
