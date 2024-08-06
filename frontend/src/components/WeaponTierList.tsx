'use client';

import Link from 'next/link';
import { useState, useEffect, useMemo } from 'react';

import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';

import { OptimizationResult } from './types';
import { Button } from './ui/button';
import { useWeaponData } from './WeaponDataContext';

const SNIPER_WEAPONS = new Set([
  'AfterglowSword',
  'Belief',
  'DifferentDream',
  'ForrestGaze',
  'PiercingLight',
  'RecipientUnknown',
  'SupermoonZ15'
]);

const WeaponTierList: React.FC = () => {
  const { weaponData, isLoading, error, refreshData } = useWeaponData();
  const [isValby, setIsValby] = useState(false);
  const [tiers, setTiers] = useState<{ [key: string]: { tier: string; dps: number } }>({});

  const filteredData = useMemo(() => {
    if (!weaponData) return {};
    return Object.entries(weaponData).reduce((acc, [weaponName, data]) => {
      const relevantKey = Object.keys(data).find(key => 
        key.startsWith('1_') && key.endsWith(isValby ? 'valby' : 'noValby')
      );
      if (relevantKey && !isSniper(weaponName)) {
        acc[weaponName] = data[relevantKey];
      }
      return acc;
    }, {} as { [key: string]: OptimizationResult });
  }, [weaponData, isValby]);

  function isSniper(weaponName: string): boolean {
    return SNIPER_WEAPONS.has(weaponName);
  }

  useEffect(() => {
    setTiers(assignTiers(filteredData));
  }, [filteredData]);

  const assignTiers = (data: { [key: string]: OptimizationResult }): { [key: string]: { tier: string; dps: number } } => {
    const values: number[] = Object.values(data).map(wd => wd.max_dps);
    const n = values.length;

    if (n === 0) return {};

    const mean = values.reduce((sum, val) => sum + val, 0) / n;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / n;
    const stdDev = Math.sqrt(variance);

    return Object.entries(data).reduce((acc, [name, weaponData]) => {
      const zScore = (weaponData.max_dps - mean) / stdDev;
      let tier: string;
      if (zScore > 2.0) tier = 'S';
      else if (zScore > 1.0) tier = 'A';
      else if (zScore > 0.0) tier = 'B';
      else if (zScore > -1.0) tier = 'C';
      else if (zScore > -2.0) tier = 'D';
      else tier = 'F';
      acc[name] = { tier, dps: Math.floor(weaponData.max_dps) };
      return acc;
    }, {} as { [key: string]: { tier: string; dps: number } });
  };

  const tierOrder = ['S', 'A', 'B', 'C', 'D', 'F'];

  if (isLoading) {
    return (
      <div className="p-4">
        <div className="mt-6 p-4 bg-card rounded-lg">
          <h2 className="text-xl font-bold mb-2 text-card-foreground text-center animate-pulse">
            Getting weapon data from database...
          </h2>
          <p className="text-center mb-2">This may take a moment if the server was inactive.</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4">
        <div className="mt-6 p-4 bg-card rounded-lg text-red-500">
          <h2 className="text-xl font-bold mb-2 text-center">Error</h2>
          <p className="text-center mb-2">{error}</p>
          <div className="flex justify-center">
            <Button onClick={() => refreshData()}>Retry</Button>
          </div>
        </div>
      </div>
    );
  }

  if (!weaponData) return <div>No weapon data available</div>;

  return (
    <div className="p-4">
      <div className="flex justify-between items-center mb-4">
        <div className="flex items-center space-x-2">
          <Switch
            id="valby-mode"
            checked={isValby}
            onCheckedChange={setIsValby}
          />
          <Label htmlFor="valby-mode">Valby Mode</Label>
        </div>
        <Link href="/">
          <Button variant="outline">Back to Optimizer</Button>
        </Link>
      </div>
      
      {tierOrder.map(tier => (
        <div key={tier} className="mb-4">
          <h2 className="text-2xl font-bold mb-2">{tier}</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
            {Object.entries(tiers)
              .filter(([, weaponInfo]) => weaponInfo.tier === tier)
              .sort(([, a], [, b]) => b.dps - a.dps)
              .map(([weaponName, weaponInfo]) => (
                <Link 
                  key={weaponName} 
                  href={`/?weapon=${encodeURIComponent(weaponName)}&valby=${isValby}&hitChance=1`}
                  className="bg-card text-card-foreground p-2 rounded hover:bg-accent hover:text-accent-foreground cursor-pointer"
                >
                  <div className="flex justify-between items-center">
                    <span>{weaponName}</span>
                    <span className="font-bold">{weaponInfo.dps}</span>
                  </div>
                </Link>
              ))
            }
          </div>
        </div>
      ))}
    </div>
  );
};

export default WeaponTierList;