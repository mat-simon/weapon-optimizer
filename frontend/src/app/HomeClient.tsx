'use client';

import dynamic from 'next/dynamic';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { Suspense } from 'react';

import { ChangelogButton } from '@/components/ChangelogButton';
import { Button } from '@/components/ui/button';

const DynamicWeaponOptimizer = dynamic(() => import('@/components/WeaponOptimizer'), {
  ssr: false,
});

export default function HomeClient() {
  const searchParams = useSearchParams();
  const weaponFromUrl = searchParams.get('weapon');
  const valbyFromUrl = searchParams.get('valby') === 'true';
  const hitChanceFromUrl = searchParams.get('hitChance');

  return (
    <>
      <ChangelogButton />
      <div className="w-full max-w-md">
        <div className="flex justify-between items-center mb-4">
          <h1 className="text-2xl font-bold">Weapon Optimizer</h1>
          <Link href="/tier-list">
            <Button variant="outline">View Tier List</Button>
          </Link>
        </div>
        <Suspense fallback={<div>Loading...</div>}>
          <DynamicWeaponOptimizer 
            initialWeapon={weaponFromUrl} 
            initialValby={valbyFromUrl} 
            initialHitChance={hitChanceFromUrl}
          />
        </Suspense>
      </div>
    </>
  );
}