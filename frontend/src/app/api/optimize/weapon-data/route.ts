import { NextResponse } from 'next/server';

import clientPromise from '@/lib/mongodb';
import { OptimizationResult } from '@/lib/optimization';

export async function GET() {
    try {
        const client = await clientPromise;
        const db = client.db("weapon_optimizer");
        const collection = db.collection<OptimizationResult>("weapon_results");
        const results = await collection.find({}).toArray();
        const weaponData: { [key: string]: { none: { dps: number }, valby: { dps: number }, enzo: { dps: number } } } = {};
        results.forEach((result: OptimizationResult) => {
            const key = result.weapon;
            if (!weaponData[key]) {
                weaponData[key] = { none: { dps: 0 }, valby: { dps: 0 }, enzo: { dps: 0 } };
            }
            if (result.valby) {
                weaponData[key].valby.dps = result.max_dps;
            } else if (result.enzo) {
                weaponData[key].enzo.dps = result.max_dps;
            } else {
                weaponData[key].none.dps = result.max_dps;
            }
        });
        return NextResponse.json(weaponData);
    } catch (error) {
        console.error('Failed to fetch weapon data:', error);
        return NextResponse.json({ error: 'Failed to fetch weapon data' }, { status: 500 });
    }
}