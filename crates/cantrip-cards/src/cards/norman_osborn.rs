use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::mayhem;
use cantrip_mechanics::triggers::deals_combat_damage_to_player;
use cantrip_mechanics::connive;
use cantrip_oracle as oracle;

fn connives(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    connive(g, item.object, item.object)
}

/// "Spells you cast from your graveyard cost {2} less to cast" (CR 601.2f): a spell on the stack that was cast from a
/// graveyard, or a card in one that would be.
fn graveyard_spells_cost_two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> i8 {
    let Costed::Spell(spell) = costed else { return 0 };
    let obj = q.obj(spell);
    if player == q.controller(source) && (obj.cast_from == Some(ZoneKind::Graveyard) || obj.zone.is(ZoneKind::Graveyard)) {
        -2
    } else {
        0
    }
}

pub static BACK: CardDef = CardDef {
    cost_changes: &[graveyard_spells_cost_two_less],
    granted_ways: &[mayhem::GRANTED],
    ..oracle::GREEN_GOBLIN
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Norman Osborn deals combat damage to a player, he connives.",
            listens: EventMask::DAMAGE,
            matches: deals_combat_damage_to_player,
            effect: connives,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{U}{B}{R}: Transform Norman Osborn. Activate only as a sorcery.",
            cost: Cost { mana: mana(1, 0, 1, 1, 1, 0), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: effects::transform_self,
            ..ActivatedDef::BLANK
        }),
    ],
    back: Some(&BACK),
    ..oracle::NORMAN_OSBORN
};
