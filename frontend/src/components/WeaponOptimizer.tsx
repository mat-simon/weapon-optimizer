'use client';

import { Check, ChevronsUpDown } from "lucide-react"
import { useState, useEffect, useCallback } from 'react'

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
// import { Progress } from '@/components/ui/progress'
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"

interface Weapon {
  name: string;
  image: string;
}

interface OptimizationResult {
  max_dps: number;
  best_rolls: Array<{
    roll_type: string;
    value: number;
  }>;
  best_modules: Array<{
    name: string;
    module_type: string;
    effects: Array<{
      effect_type: string;
      value: number;
    }>;
  }>;
}

interface OptimizationConfig {
  valby: boolean;
  gley: boolean;
  gley_duration: number;
}

const placeholderImage = 'https://via.placeholder.com/40';
const hitChances = [0, 0.25, .33, 0.5, .67, 0.75, 1];
const CACHE_KEY = 'weaponOptimizerCache';

export default function WeaponOptimizer() {
  const [isLoading, setIsLoading] = useState(true);
  const [weapons, setWeapons] = useState<Weapon[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedWeapon, setSelectedWeapon] = useState<Weapon | null>(null);
  const [open, setOpen] = useState(false)
  const [hitChance, setHitChance] = useState<string>('1');
  const [result, setResult] = useState<OptimizationResult | null>(null);
  const [isCalculating, setIsCalculating] = useState(false);
  const [config, setConfig] = useState<OptimizationConfig>({ valby: false, gley: false, gley_duration: 9.0});
  const [cache, setCache] = useState<Record<string, OptimizationResult>>({});
  // const [progress, setProgress] = useState(0);

  useEffect(() => {
    const checkServerReady = async () => {
      try {
        const response = await fetch('http://localhost:8080/server-ready');
        if (response.ok) {
          setIsLoading(false);
          fetchWeapons();
        } else {
          setTimeout(checkServerReady, 1000);
        }
      } catch (error) {
        console.error("Error checking server status:", error);
        setTimeout(checkServerReady, 5000);
      }
    };

    checkServerReady();
  }, []);

  useEffect(() => {
    // Load cache from localStorage on component mount
    const savedCache = localStorage.getItem(CACHE_KEY);
    if (savedCache) {
      setCache(JSON.parse(savedCache));
    }
  }, []);

  useEffect(() => {
    // Save cache to localStorage whenever it changes
    localStorage.setItem(CACHE_KEY, JSON.stringify(cache));
  }, [cache]);

  const fetchWeapons = async () => {
    console.log("Fetching weapons...");
    const response = await fetch('http://localhost:8080/weapons');
    const data = await response.json();
    console.log("Weapons fetched:", data);
    setWeapons(data.map((name: string) => ({ name, image: placeholderImage })));
  };

  const handleOptimize = useCallback(async () => {
    if (!selectedWeapon) return;

    const cacheKey = `${selectedWeapon.name}_${hitChance}_${config.valby}_${config.gley}`;
    if (cache[cacheKey]) {
      setResult(cache[cacheKey]);
      return;
    }

    setIsCalculating(true);
    setResult(null);

    try {
        const response = await fetch('http://localhost:8080/optimize', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ 
                weapon: selectedWeapon.name, 
                weak_point_hit_chance: parseFloat(hitChance),
                valby: config.valby,
                gley: config.gley,
            }),
        });

        if (response.status === 503) {
          // Request is queued
          const retryAfter = response.headers.get('Retry-After');
          if (retryAfter) {
            const delay = parseInt(retryAfter) * 1000;
            await new Promise(resolve => setTimeout(resolve, delay));
            return handleOptimize(); // Retry after delay
          }
        }

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        setResult(result);
        setCache(prevCache => ({...prevCache, [cacheKey]: result}));
    } catch (error) {
        console.error("There was a problem with the optimization:", error);
        // Handle the error appropriately in your UI
    } finally {
        setIsCalculating(false);
    }
  }, [selectedWeapon, hitChance, config, cache]);

  const filteredWeapons = weapons.filter(weapon => 
    weapon.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="space-y-6">
      {isLoading ? (
        <div className="text-center">
          Preparing Server...
        </div>
      ) : (
        <>
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
                    setConfig(prev => ({ ...prev, valby: true, gley: false }));
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
                id="gley"
                checked={config.gley}
                onCheckedChange={(checked) => {
                  if (checked) {
                    setConfig(prev => ({ ...prev, gley: true, valby: false }));
                  } else {
                    setConfig(prev => ({ ...prev, gley: false }));
                  }
                }}
              />
              <label 
                htmlFor="gley" 
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Gley Infinite Ammo Build
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

          {isCalculating ? (
            <div className="mt-6 p-4 bg-card rounded-lg">
              <h2 className="text-xl font-bold mb-2 text-card-foreground text-center animate-pulse">
                Calculating...
              </h2>
              <p className="text-center mb-2">Simulating weapon combinations</p>
              {/* <Progress value={progress} className="w-full" />
              <p className="text-center mt-2">{progress}% complete</p> */}
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
        </>
      )}
    </div>
  );
}