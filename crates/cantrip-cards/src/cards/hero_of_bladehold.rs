use cantrip_api::*;
use cantrip_mechanics::battle_cry;
use cantrip_mechanics::combat::{choose_attack_target, create_attacking};
use cantrip_mechanics::tokens::SOLDIER;
use cantrip_oracle as oracle;

/// Two 1/1 white Soldier tokens, tapped and attacking (CR 508.4): their controller chooses what each one attacks as it
/// enters, a player, a planeswalker or a battle, asked only when there is a choice. The ones that attack the same thing
/// are created together (CR 101.4).
fn soldiers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let soldier = g.card_id(&SOLDIER);
    let mut chosen: Vec<Target> = Vec::new();
    for _ in 0..2 {
        match choose_attack_target(g, item.controller, item.object)? {
            Some(target) => chosen.push(target),
            None => break,
        }
    }
    create_attacking(g, item.controller, soldier, &chosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        battle_cry::TRIGGER,
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Hero of Bladehold attacks, create two 1/1 white Soldier creature tokens that are tapped and attacking.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::this_attacks,
            effect: soldiers,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::HERO_OF_BLADEHOLD
};

pub static EXTRA: &[&CardDef] = &[&SOLDIER];
