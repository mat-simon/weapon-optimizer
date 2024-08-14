'use client';

import Link from 'next/link';
import { useState, useEffect, useMemo } from 'react';

import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group"

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

const SniperTierList: React.FC = () => {
  const { weaponData, isLoading, error, refreshData } = useWeaponData();
  const [mode, setMode] = useState<'none' | 'valby' | 'enzo'>('none');
  const [tiers, setTiers] = useState<{ [key: string]: { tier: string; dps: number } }>({});

  const filteredData = useMemo(() => {
    console.log('WeaponData in filteredData:', weaponData);
    if (!weaponData) return {};
    return Object.entries(weaponData).reduce((acc, [weaponName, weaponModes]) => {
      if (isSniper(weaponName)) {
        const relevantKey = Object.keys(weaponModes).find(key => {
          const [hitChance, modeType] = key.split('_');
          return hitChance === '1' && modeType === mode;
        });
        
        if (relevantKey && weaponModes[relevantKey]) {
          acc[weaponName] = { dps: weaponModes[relevantKey].max_dps };
        }
      }
      return acc;
    }, {} as { [key: string]: { dps: number } });
  }, [weaponData, mode]);

  function isSniper(weaponName: string): boolean {
    return SNIPER_WEAPONS.has(weaponName);
  }

  useEffect(() => {
    console.log('FilteredData:', filteredData);
    setTiers(assignTiers(filteredData));
  }, [filteredData]);

  const assignTiers = (data: { [key: string]: { dps: number } }): { [key: string]: { tier: string; dps: number } } => {
    console.log('Assigning tiers for data:', data);
    const values: number[] = Object.values(data).map(wd => wd.dps);
    const n = values.length;

    if (n === 0) return {};

    const mean = values.reduce((sum, val) => sum + val, 0) / n;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / n;
    const stdDev = Math.sqrt(variance);

    const result = Object.entries(data).reduce((acc, [name, weaponData]) => {
      const zScore = (weaponData.dps - mean) / stdDev;
      let tier: string;
      if (zScore > 2.0) tier = 'S';
      else if (zScore > 1.0) tier = 'A';
      else if (zScore > 0.0) tier = 'B';
      else if (zScore > -1.0) tier = 'C';
      else if (zScore > -2.0) tier = 'D';
      else tier = 'F';
      acc[name] = { tier, dps: Math.floor(weaponData.dps) };
      return acc;
    }, {} as { [key: string]: { tier: string; dps: number } });

    console.log('Assigned tiers:', result);
    return result;
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
        <RadioGroup value={mode} onValueChange={(value: 'none' | 'valby' | 'enzo') => setMode(value)} className="flex space-x-4">
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="none" id="none" />
            <Label htmlFor="none">Normal</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="valby" id="valby" />
            <Label htmlFor="valby">Valby</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="enzo" id="enzo" />
            <Label htmlFor="enzo">Enzo</Label>
          </div>
        </RadioGroup>
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
                  href={`/?weapon=${encodeURIComponent(weaponName)}${mode !== 'none' ? `&${mode}=true` : ''}&hitChance=1`}
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

export default SniperTierList;