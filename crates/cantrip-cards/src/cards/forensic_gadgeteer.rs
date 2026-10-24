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
/// Asked of activated abilities alone (`Costed::Ability`): never of a spell, nor of a special action (CR 116.1).
fn artifact_abilities(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Ability(permanent, ..) = costed else { return CostDelta::NONE };
    let you = q.controller(source);
    if player == you && q.controller(permanent) == you && q.is_on_battlefield(permanent) && q.chars(permanent).is(CardType::Artifact) {
        CostDelta::less_to_one(1)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::SPELL_CAST, you_cast_an_artifact_spell), effect, "Whenever you cast an artifact spell, investigate.")],
    cost_changes: &[artifact_abilities],
    ..oracle::FORENSIC_GADGETEER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
