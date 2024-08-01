import { ChangelogButton } from '@/components/ChangelogButton';
import WeaponOptimizer from '@/components/WeaponOptimizer';

export default function Home() {
  return (
    <main className="container mx-auto p-4 min-h-screen flex flex-col items-center relative">
      <ChangelogButton />
      <div className="w-full max-w-md">
        <h1 className="text-2xl font-bold mb-4 text-center">Weapon Optimizer</h1>
        <WeaponOptimizer />
      </div>
    </main>
  );
}