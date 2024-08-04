import dynamic from 'next/dynamic';

const DynamicHomeClient = dynamic(() => import('./HomeClient'), {
  ssr: false,
});

export default function Home() {
  return (
    <main className="container mx-auto p-4 min-h-screen flex flex-col items-center relative">
      <DynamicHomeClient />
    </main>
  );
}