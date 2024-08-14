'use client';

import React, { createContext, useState, useContext, useEffect } from 'react';

import { WeaponDataMap, CachedWeaponData, OptimizationResult } from './types';

const API_URL = 'https://weapon-optimizer-api.onrender.com';
const CACHE_KEY = 'weaponDataCache';
const CACHE_TIMESTAMP_KEY = 'weaponDataCacheTimestamp';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24 hours

interface WeaponDataContextType {
  weaponData: WeaponDataMap | null;
  cachedData: CachedWeaponData | null;
  isLoading: boolean;
  error: string | null;
  refreshData: () => Promise<void>;
  clearCacheAndFetch: (target: string) => Promise<void>;
  clearLocalCache: () => void;
  updateCache: (key: string, data: OptimizationResult) => void;
}

const WeaponDataContext = createContext<WeaponDataContextType | undefined>(undefined);

export const WeaponDataProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
  const [weaponData, setWeaponData] = useState<WeaponDataMap | null>(null);
  const [cachedData, setCachedData] = useState<CachedWeaponData | null>(null);
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
      setWeaponData(data);
      localStorage.setItem(CACHE_KEY, JSON.stringify(data));
      localStorage.setItem(CACHE_TIMESTAMP_KEY, Date.now().toString());
      setCachedData(data as unknown as CachedWeaponData);
      setError(null);
      console.log('Fetched and cached new weapon data:', data);
    } catch (err) {
      setError('Failed to load weapon data');
      console.error('Error fetching weapon data:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const loadCachedData = () => {
    const cachedData = localStorage.getItem(CACHE_KEY);
    const timestamp = localStorage.getItem(CACHE_TIMESTAMP_KEY);
    console.log('Attempting to load cached data');
    console.log('Cached data:', cachedData);
    console.log('Timestamp:', timestamp);
    if (cachedData && timestamp) {
      const parsedData = JSON.parse(cachedData) as WeaponDataMap;
      if (Date.now() - parseInt(timestamp) < CACHE_DURATION) {
        setWeaponData(parsedData);
        setCachedData(parsedData as unknown as CachedWeaponData);
        console.log('Loaded cached weapon data:', parsedData);
        return true;
      }
    }
    console.log('No valid cached data found');
    return false;
  };

  useEffect(() => {
    if (!loadCachedData()) {
      fetchWeaponData();
    } else {
      setIsLoading(false);
    }
  }, []);

  const refreshData = async () => {
    clearLocalCache();
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
      clearLocalCache();
      await fetchWeaponData();
    } catch (err) {
      console.error('Error clearing cache and fetching new data:', err);
      setError('Failed to clear cache and fetch new data');
    }
  };

  const clearLocalCache = () => {
    localStorage.removeItem(CACHE_KEY);
    localStorage.removeItem(CACHE_TIMESTAMP_KEY);
    setCachedData(null);
    console.log('Local cache cleared');
  };

  const updateCache = (key: string, data: OptimizationResult) => {
    setCachedData(prevData => {
      const newData = prevData ? { ...prevData, [key]: data } : { [key]: data };
      localStorage.setItem(CACHE_KEY, JSON.stringify(newData));
      console.log(`Cache updated for key: ${key}`, newData);
      return newData;
    });
  };

  return (
    <WeaponDataContext.Provider value={{ 
      weaponData, 
      cachedData,
      isLoading, 
      error, 
      refreshData, 
      clearCacheAndFetch,
      clearLocalCache,
      updateCache
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