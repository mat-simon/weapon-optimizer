import dynamic from 'next/dynamic';
import { Suspense } from 'react';

const DynamicWeaponTierList = dynamic(() => import('@/components/WeaponTierList'), {
  ssr: false,
});

export default function TierListPage() {
  return (
    <main className="container mx-auto p-4 min-h-screen flex flex-col items-center relative">
      <div className="w-full max-w-6xl">
        <h1 className="text-2xl font-bold mb-4">Weapon Tier List</h1>
        <Suspense fallback={<div>Loading...</div>}>
          <DynamicWeaponTierList />
        </Suspense>
      </div>
    </main>
  );
}