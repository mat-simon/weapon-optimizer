use crate::weapons::{Module, ModuleType, ModuleBonusType, ModuleEffect};

pub fn get_modules() -> Vec<Module> {
    vec![
        Module {
            name: "Action and Reaction".to_string(),
            module_type: ModuleType::Atk,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.61 }],
        },
        Module {
            name: "Ele Enhancement".to_string(),
            module_type: ModuleType::EleEnhancement,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::EleEnhancement, value: 0.30 }],
        },
        Module {
            name: "Rifling Reinforcement".to_string(),
            module_type: ModuleType::None,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.32 }],
        },
        Module {
            name: "Better Insight".to_string(),
            module_type: ModuleType::None,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.39 }],
        },
        Module {
            name: "Better Concentration".to_string(),
            module_type: ModuleType::None,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.748 }],
        },
        Module {
            name: "Weak Point Sight".to_string(),
            module_type: ModuleType::None,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.35 }],
        },
        Module {
            name: "Expand Weapon Charge".to_string(),
            module_type: ModuleType::None,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::RoundsPerMagazine, value: 0.49 }],
        },
        Module {
            name: "Recycling Genius".to_string(),
            module_type: ModuleType::None,
            effects: vec![ModuleEffect { effect_type: ModuleBonusType::ReloadTime, value: -0.3 }],
        },
        Module {
            name: "Anti-matter Round".to_string(),
            module_type: ModuleType::Atk,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.26 },
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.065 },
            ],
        },
        Module {
            name: "Pinpoint Shot".to_string(),
            module_type: ModuleType::Atk,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.26 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.02 },
            ],
        },
        Module {
            name: "Sharpshooter".to_string(),
            module_type: ModuleType::Atk,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.26 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.015 },
            ],
        },
        Module {
            name: "Slow Art".to_string(),
            module_type: ModuleType::Atk,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.62 },
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: -0.25 },
            ],
        },
        Module {
            name: "Bullet Rain".to_string(),
            module_type: ModuleType::FireRate,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.01 },
            ],
        },
        Module {
            name: "Rapid Fire Insight".to_string(),
            module_type: ModuleType::FireRate,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.015 },
            ],
        },
        Module {
            name: "Fire Rate Up".to_string(),
            module_type: ModuleType::FireRate,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: 0.25 }],
        },
        Module {
            name: "Weak Point Quick Fire".to_string(),
            module_type: ModuleType::FireRate,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.02 },
            ],
        },
        Module {
            name: "Focus Fire".to_string(),
            module_type: ModuleType::WeakPointStrike,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.065 },
            ],
        },
        Module {
            name: "Weak Point Insight".to_string(),
            module_type: ModuleType::WeakPointStrike,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.015 },
            ],
        },
        Module {
            name: "Weak Point Detection".to_string(),
            module_type: ModuleType::WeakPointStrike,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.01 },
            ],
        },
        Module {
            name: "Have Aiming".to_string(),
            module_type: ModuleType::WeakPointStrike,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.40 }],
        },
        Module {
            name: "Insight Focus".to_string(),
            module_type: ModuleType::Crit,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.145 },
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.065 },
            ],
        },
        Module {
            name: "Adventurer".to_string(),
            module_type: ModuleType::Crit,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.145 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.02 },
            ],
        },
        Module {
            name: "Edging Shot".to_string(),
            module_type: ModuleType::Crit,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.43 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: -0.15 },
            ],
        },
        Module {
            name: "Marksman".to_string(),
            module_type: ModuleType::Crit,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.145 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.01 },
            ],
        },
        Module {
            name: "Concentration Priority".to_string(),
            module_type: ModuleType::CritDamage,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 1.2 },
                ModuleEffect { effect_type: ModuleBonusType::ReloadTime, value: 0.3 },
            ],
        },
        Module {
            name: "Fatal Critical".to_string(),
            module_type: ModuleType::CritDamage,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.34 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.01 },
            ],
        },
        Module {
            name: "Commando Marksmanship".to_string(),
            module_type: ModuleType::CritDamage,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.34 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.01 },
            ],
        },
        Module {
            name: "Target Detection".to_string(),
            module_type: ModuleType::CritDamage,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.34 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.02 },
            ],
        },
        Module {
            name: "Fire Rate Concentration".to_string(),
            module_type: ModuleType::FireRate,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: 0.20 },
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.065 },
            ],
        },
        Module {
            name: "Concentrate Support Ammo".to_string(),
            module_type: ModuleType::RoundsPerMagazine,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::RoundsPerMagazine, value: 0.3 },
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.065 },
            ],
        },
        Module {
            name: "Insight Support Ammo".to_string(),
            module_type: ModuleType::RoundsPerMagazine,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::RoundsPerMagazine, value: 0.3 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.015 },
            ],
        },
        Module {
            name: "Magazine Compulsive".to_string(),
            module_type: ModuleType::RoundsPerMagazine,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::RoundsPerMagazine, value: 0.39 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: -0.1 },
            ],
        },
        Module {
            name: "Weapon Tuning".to_string(),
            module_type: ModuleType::RoundsPerMagazine,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::RoundsPerMagazine, value: 0.3 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.01 },
            ],
        },
        Module {
            name: "Maximize Weight Balance".to_string(),
            module_type: ModuleType::RoundsPerMagazine,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::RoundsPerMagazine, value: 0.3 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.02 },
            ],
        },
        Module {
            name: "Consume Magazines".to_string(),
            module_type: ModuleType::ReloadTime,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::ReloadTime, value: -0.25 },
                ModuleEffect { effect_type: ModuleBonusType::WeakPointDamage, value: 0.02 },
            ],
        },
        Module {
            name: "Reload Insight".to_string(),
            module_type: ModuleType::ReloadTime,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::ReloadTime, value: -0.25 },
                ModuleEffect { effect_type: ModuleBonusType::Crit, value: 0.03 },
            ],
        },
        Module {
            name: "Reload Expert".to_string(),
            module_type: ModuleType::ReloadTime,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::ReloadTime, value: -0.25 },
                ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.01 },
            ],
        },
        Module {
            name: "Reload Focus".to_string(),
            module_type: ModuleType::ReloadTime,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::ReloadTime, value: -0.25 },
                ModuleEffect { effect_type: ModuleBonusType::CritDamage, value: 0.065 },
            ],
        },
        Module {
            name: "Ele Gunbarrel".to_string(),
            module_type: ModuleType::Gunbarrel,
            effects: vec![
                ModuleEffect { effect_type: ModuleBonusType::EleMult, value: 0.80 },
                ModuleEffect { effect_type: ModuleBonusType::FireRate, value: -0.25 }
            ],
        },
        // Module {
        //     name: "Firing Fiesta".to_string(),
        //     module_type: ModuleType::SpecialMod,
        //     effects: vec![
        //         ModuleEffect { effect_type: ModuleBonusType::FiringFiesta, value: 1.0 }],
        // },
        // Module {
        //     name: "Sweeping Squad".to_string(),
        //     module_type: ModuleType::SpecialMod,
        //     effects: vec![
        //         ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.318 }],
        // },
        // Module {
        //     name: "Ele Conductor".to_string(),
        //     module_type: ModuleType::SpecialMod,
        //     effects: vec![ModuleEffect { effect_type: ModuleBonusType::Atk, value: 0.26 }],
        // },
    ]
}