import './globals.css'

import { WeaponDataProvider } from '@/components/WeaponDataContext'

export const metadata = {
  title: 'Weapon Optimizer',
  description: 'Optimize your weapon loadout',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <WeaponDataProvider>
          {children}
        </WeaponDataProvider>
      </body>
    </html>
  )
}