'use client';

import { useState } from 'react';

import SniperTierList from '@/components/SniperTierList';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import WeaponTierList from '@/components/WeaponTierList';

export default function TierListPage() {
  const [currentTab, setCurrentTab] = useState("regular");

  return (
    <main className="container mx-auto p-4 min-h-screen flex flex-col items-center relative">
      <div className="w-full max-w-6xl">
        <h1 className="text-2xl font-bold mb-4">Weapon Tier List</h1>
        
        <Tabs value={currentTab} onValueChange={setCurrentTab}>
          <TabsList>
            <TabsTrigger value="regular">Weapons</TabsTrigger>
            <TabsTrigger value="sniper">Sniper Rifles</TabsTrigger>
          </TabsList>
          <TabsContent value="regular">
            <WeaponTierList />
          </TabsContent>
          <TabsContent value="sniper">
            <SniperTierList />
          </TabsContent>
        </Tabs>
      </div>
    </main>
  );
}