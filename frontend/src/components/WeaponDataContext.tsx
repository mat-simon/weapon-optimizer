'use client';

import React, { createContext, useState, useContext, useEffect } from 'react';

import { WeaponDataMap, OptimizationResult } from './types';

const API_URL = 'https://weapon-optimizer-api.onrender.com';

interface WeaponDataContextType {
  weaponData: WeaponDataMap | null;
  isLoading: boolean;
  error: string | null;
  fetchWeaponData: () => Promise<void>;
  optimizeWeapon: (weapon: string, config: string) => Promise<OptimizationResult | null>;
}

const WeaponDataContext = createContext<WeaponDataContextType | undefined>(undefined);

export const WeaponDataProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
  const [weaponData, setWeaponData] = useState<WeaponDataMap | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchWeaponData = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_URL}/weapon-data`);
      if (!response.ok) throw new Error('Failed to fetch weapon data');
      const data: WeaponDataMap = await response.json();
      setWeaponData(data);
    } catch (err) {
      setError('Failed to load weapon data');
      // console.error('Error fetching weapon data:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const optimizeWeapon = async (weapon: string, config: string): Promise<OptimizationResult | null> => {
    try {
      const response = await fetch(`${API_URL}/optimize`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ weapon, ...JSON.parse(config) }),
      });
      if (!response.ok) throw new Error('Optimization failed');
      return await response.json();
    } catch (err) {
      // console.error('Error optimizing weapon:', err);
      return null;
    }
  };

  useEffect(() => {
    fetchWeaponData();
  }, []);

  return (
    <WeaponDataContext.Provider value={{ 
      weaponData, 
      isLoading, 
      error, 
      fetchWeaponData,
      optimizeWeapon
    }}>
      {children}
    </WeaponDataContext.Provider>
  );
};

export const useWeaponData = () => {
  const context = useContext(WeaponDataContext);
  if (context === undefined) {
    throw new Error('useWeaponData must be used within a WeaponDataProvider');
  }
  return context;
};