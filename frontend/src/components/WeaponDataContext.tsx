'use client';

import React, { createContext, useState, useContext, useEffect } from 'react';

import { WeaponDataMap } from './types';

const API_URL = 'https://weapon-optimizer-api.onrender.com';

interface WeaponDataContextType {
  weaponData: WeaponDataMap | null;
  isLoading: boolean;
  error: string | null;
  refreshData: () => Promise<void>;
}

const WeaponDataContext = createContext<WeaponDataContextType | undefined>(undefined);

export const WeaponDataProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
  const [weaponData, setWeaponData] = useState<WeaponDataMap | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchWeaponData = async () => {
    setIsLoading(true);
    try {
      const response = await fetch(`${API_URL}/weapon-data`);
      if (!response.ok) {
        throw new Error('Failed to fetch weapon data');
      }
      const data: WeaponDataMap = await response.json();
      console.log('Fetched weapon data:', data);
      setWeaponData(data);
      setError(null);
    } catch (err) {
      setError('Failed to load weapon data');
      console.error('Error fetching weapon data:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchWeaponData();
  }, []);

  const refreshData = async () => {
    await fetchWeaponData();
  };

  return (
    <WeaponDataContext.Provider value={{ weaponData, isLoading, error, refreshData }}>
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