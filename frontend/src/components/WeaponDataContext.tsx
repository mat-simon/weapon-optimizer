'use client';

import React, { createContext, useState, useContext, useEffect } from 'react';

import { WeaponDataMap } from './types';

const API_URL = 'https://weapon-optimizer-api.onrender.com';
const CACHE_KEY = 'weaponDataCache';

interface WeaponDataContextType {
  weaponData: WeaponDataMap | null;
  isLoading: boolean;
  error: string | null;
  refreshData: () => Promise<void>;
  clearCacheAndFetch: (target: string) => Promise<void>;
}

const WeaponDataContext = createContext<WeaponDataContextType | undefined>(undefined);

export const WeaponDataProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
  const [weaponData, setWeaponData] = useState<WeaponDataMap | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchWeaponData = async () => {
    setIsLoading(true);
    try {
      // Check localStorage first
      const cachedData = localStorage.getItem(CACHE_KEY);
      if (cachedData) {
        const parsedData = JSON.parse(cachedData);
        setWeaponData(parsedData);
        setIsLoading(false);
        setError(null);
        console.log('Using cached weapon data');
        return;
      }

      const response = await fetch(`${API_URL}/weapon-data`);
      if (!response.ok) {
        throw new Error('Failed to fetch weapon data');
      }
      const data: WeaponDataMap = await response.json();
      console.log('Fetched weapon data:', data);
      setWeaponData(data);
      // Cache the data
      localStorage.setItem(CACHE_KEY, JSON.stringify(data));
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
    // Clear the cache before fetching new data
    localStorage.removeItem(CACHE_KEY);
    await fetchWeaponData();
  };

  const clearCacheAndFetch = async (target: string) => {
    try {
      const response = await fetch(`${API_URL}/clear-cache-and-fetch?target=${target}`, {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error('Failed to clear cache and fetch new data');
      }
      // Clear local cache and refresh data
      localStorage.removeItem(CACHE_KEY);
      await fetchWeaponData();
    } catch (err) {
      console.error('Error clearing cache and fetching new data:', err);
      setError('Failed to clear cache and fetch new data');
    }
  };

  return (
    <WeaponDataContext.Provider value={{ 
      weaponData, 
      isLoading, 
      error, 
      refreshData, 
      clearCacheAndFetch
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