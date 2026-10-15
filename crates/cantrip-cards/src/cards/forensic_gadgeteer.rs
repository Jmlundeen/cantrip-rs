use cantrip_api::*;
use cantrip_mechanics::tokens::{self, investigate};
use cantrip_oracle as oracle;

/// "Whenever you cast an artifact spell" (CR 603.2).
fn you_cast_an_artifact_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && q.chars(*spell).is(CardType::Artifact) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    investigate(g, item.controller)
}

/// "Activated abilities of artifacts you control cost {1} less to activate. This effect can't reduce the mana in that
/// cost to less than one mana" (CR 601.2f, 118.7).
fn artifact_abilities(q: &dyn Query, source: ObjectId, player: PlayerId, permanent: ObjectId, _: AbilityRef) -> u8 {
    let you = q.controller(source);
    if player == you && q.controller(permanent) == you && q.is_on_battlefield(permanent) && q.chars(permanent).is(CardType::Artifact) {
        1
    } else {
        0
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast an artifact spell, investigate.",
        listens: EventMask::SPELL_CAST,
        matches: you_cast_an_artifact_spell,
        effect,
        ..TriggeredDef::BLANK
    })],
    ability_discounts: &[artifact_abilities],
    ..oracle::FORENSIC_GADGETEER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
