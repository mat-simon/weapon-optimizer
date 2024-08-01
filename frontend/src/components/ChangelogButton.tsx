'use client';

import { useState } from 'react';

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

interface ChangelogEntry {
  version: string;
  date: string;
  changes: string[];
}

const changelog: ChangelogEntry[] = [
  {
    version: "1.0",
    date: "2024-07-29",
    changes: [
      "Initial release",
    ]
  },
  // Add more entries as you update your app
];

export function ChangelogButton() {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" className="absolute top-4 right-4">
          Changelog
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-[425px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Changelog</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          {changelog.map((entry, index) => (
            <div key={index}>
              <h3 className="text-lg font-semibold">Version {entry.version} <span className="text-sm font-normal">({entry.date})</span></h3>
              <ul className="list-disc list-inside mt-2">
                {entry.changes.map((change, changeIndex) => (
                  <li key={changeIndex}>{change}</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}