'use client';

import { Check, ChevronsUpDown } from "lucide-react"
import { useState, useEffect, useCallback } from 'react'

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"

import { OptimizationResult } from './types';
import { useWeaponData } from './WeaponDataContext'

const API_URL = 'https://weapon-optimizer-api.onrender.com';
const placeholderImage = 'https://via.placeholder.com/40';
const hitChances = [0.25, 0.5, 1.0];

interface Weapon {
  name: string;
  image: string;
}

interface WeaponOptimizerProps {
  initialWeapon?: string | null;
  initialValby?: boolean;
}

interface OptimizationConfig {
  valby: boolean;
  enzo: boolean;
}

export default function WeaponOptimizer({ initialWeapon, initialValby }: WeaponOptimizerProps) {
  const [weapons, setWeapons] = useState<Weapon[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedWeapon, setSelectedWeapon] = useState<Weapon | null>(null);
  const [open, setOpen] = useState(false);
  const [hitChance, setHitChance] = useState<string>('1');
  const [result, setResult] = useState<OptimizationResult | null>(null);
  const [isCalculating, setIsCalculating] = useState(false);
  const [config, setConfig] = useState<OptimizationConfig>({ valby: false, enzo: false });
  const [error, setError] = useState<string | null>(null);
  const { weaponData, refreshData } = useWeaponData();

  useEffect(() => {
    fetchWeapons();
  }, []);

  useEffect(() => {
    if (initialWeapon && weapons.length > 0) {
      const weapon = weapons.find(w => w.name === initialWeapon);
      if (weapon) {
        setSelectedWeapon(weapon);
      }
    }
    if (initialValby !== undefined) {
      setConfig(prev => ({ ...prev, valby: initialValby, enzo: false }));
    }
  }, [initialWeapon, initialValby, weapons]);

  const fetchWeapons = async () => {
    try {
      const response = await fetch(`${API_URL}/weapons`);
      if (!response.ok) throw new Error('Failed to fetch weapons');
      const data = await response.json();
      setWeapons(data.map((name: string) => ({ name, image: placeholderImage })));
    } catch (error) {
      console.error("Error fetching weapons:", error);
      setError("Failed to fetch weapons. Please try again later.");
    }
  };

  const handleOptimize = useCallback(async () => {
    if (!selectedWeapon) return;

    setIsCalculating(true);
    setResult(null);
    setError(null);

    try {
      const cacheKey = `${hitChance}_${config.valby}`;
      let optimizationResult = weaponData?.[selectedWeapon.name]?.[cacheKey];

      if (!optimizationResult) {
        console.log('Cache miss, fetching from API');
        const response = await fetch(`${API_URL}/optimize`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ 
            weapon: selectedWeapon.name, 
            weak_point_hit_chance: parseFloat(hitChance),
            valby: config.valby,
            enzo: config.enzo,
          }),
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        optimizationResult = await response.json();
        await refreshData(); // Refresh the weapon data after getting a new result
      } else {
        console.log('Cache hit, using stored data');
      }

      if (optimizationResult) {
        console.log('Optimization result:', optimizationResult);
        setResult(optimizationResult);
      } else {
        throw new Error('No optimization result available');
      }
    } catch (error) {
      console.error("There was a problem with the optimization:", error);
      setError("Failed to optimize weapon. Please try again later.");
    } finally {
      setIsCalculating(false);
    }
  }, [selectedWeapon, hitChance, config, weaponData, refreshData]);

  const filteredWeapons = weapons.filter(weapon => 
    weapon.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="space-y-6">
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="w-full justify-center bg-primary text-primary-foreground hover:bg-accent hover:text-accent-foreground"
          >
            {selectedWeapon ? selectedWeapon.name : "Select weapon..."}
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-full p-0 bg-popover">
          <div className="p-2">
            <input
              type="text"
              placeholder="Search weapon..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full p-2 border rounded bg-input text-popover-foreground"
            />
          </div>
          <div className="max-h-60 overflow-auto">
            {filteredWeapons.length > 0 ? (
              filteredWeapons.map((weapon) => (
                <div
                  key={weapon.name}
                  className="flex items-center justify-between p-2 hover:bg-accent cursor-pointer text-popover-foreground"
                  onClick={() => {
                    setSelectedWeapon(weapon);
                    setOpen(false);
                  }}
                >
                  <div className="flex items-center">
                    <img src={weapon.image} alt={weapon.name} className="h-6 w-6 flex-shrink-0 rounded-full mr-2" />
                    {weapon.name}
                  </div>
                  {selectedWeapon?.name === weapon.name && (
                    <Check className="h-4 w-4" />
                  )}
                </div>
              ))
            ) : (
              <div className="p-2 text-popover-foreground">No weapons available</div>
            )}
          </div>
        </PopoverContent>
      </Popover>

      <div>
        <label className="block text-sm font-medium mb-1 text-foreground text-center">Weak Point Hit Chance</label>
        <ToggleGroup type="single" value={hitChance} onValueChange={(value) => value && setHitChance(value)}>
          {hitChances.map((chance) => (
            <ToggleGroupItem 
              key={chance} 
              value={chance.toString()} 
              className="w-full bg-primary text-primary-foreground data-[state=on]:bg-accent data-[state=on]:text-accent-foreground data-[state=on]:ring-2 data-[state=on]:ring-accent-foreground"
            >
              {chance * 100}%
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>

      <div className="flex space-x-4">
        <div className="flex items-center space-x-2">
          <Checkbox 
            id="valby"
            checked={config.valby}
            onCheckedChange={(checked) => {
              if (checked) {
                setConfig(prev => ({ ...prev, valby: true, enzo: false }));
              } else {
                setConfig(prev => ({ ...prev, valby: false }));
              }
            }}
          />
          <label 
            htmlFor="valby" 
            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Valby Moisture Supply
          </label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox 
            id="enzo"
            checked={config.enzo}
            onCheckedChange={(checked) => {
              if (checked) {
                setConfig(prev => ({ ...prev, enzo: true, valby: false }));
              } else {
                setConfig(prev => ({ ...prev, enzo: false }));
              }
            }}
          />
          <label 
            htmlFor="enzo" 
            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Enzo
          </label>
        </div>
      </div>

      <Button 
        onClick={handleOptimize} 
        className="w-full bg-primary text-primary-foreground hover:bg-accent hover:text-accent-foreground"
        disabled={isCalculating}
      >
        {isCalculating ? 'Optimizing...' : 'Optimize'}
      </Button>

      {error && (
        <div className="text-red-500 text-center mt-4">
          {error}
        </div>
      )}

      {isCalculating ? (
        <div className="mt-6 p-4 bg-card rounded-lg">
          <h2 className="text-xl font-bold mb-2 text-card-foreground text-center animate-pulse">
            Calculating...
          </h2>
          <p className="text-center mb-2">Simulating weapon combinations</p>
        </div>
      ) : result && (
        <div className="mt-6 p-4 bg-card rounded-lg">
          <h2 className="text-xl font-bold mb-2 text-card-foreground text-center">Results</h2>
          <p className="text-card-foreground text-center mb-4">DPS: {result.max_dps.toFixed(2)}</p>
          
          <h3 className="text-lg font-semibold mt-4 mb-2 text-card-foreground">Best Rolls:</h3>
          <div className="grid grid-cols-2 gap-2">
            {result.best_rolls.map((roll, index) => (
              <div key={index} className="bg-muted p-2 rounded text-muted-foreground">
                {roll.roll_type}: {roll.value.toFixed(2)}
              </div>
            ))}
          </div>
          
          <h3 className="text-lg font-semibold mt-4 mb-2 text-card-foreground">Best Modules:</h3>
          <div className="grid grid-cols-5 gap-2">
            {result.best_modules.map((module, index) => (
              <div key={index} className="bg-muted p-2 rounded text-center text-muted-foreground">
                <div>{module.name}</div>
                <div className="text-xs mt-1">{module.module_type}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}