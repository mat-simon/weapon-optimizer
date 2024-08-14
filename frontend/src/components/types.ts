export interface OptimizationResult {
  weapon: string;
  valby: boolean;
  enzo: boolean;
  max_dps: number;
  best_rolls: Array<{ roll_type: string; value: number }>;
  best_modules: Array<[{ name: string; module_type: string }, number]>;
}

export interface WeaponDataEntry {
  [key: string]: OptimizationResult;
}

export type WeaponDataMap = {
  [weaponName: string]: WeaponDataEntry;
};

export interface CachedWeaponData {
  [key: string]: OptimizationResult;
}