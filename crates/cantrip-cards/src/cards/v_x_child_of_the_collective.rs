use cantrip_api::*;
use cantrip_mechanics::{triggers, undaunted::undaunted};
use cantrip_oracle as oracle;

/// "When Võx enters and at the beginning of your first main phase" (CR 603.2, 603.6a): one ability with two trigger events.
fn enters_or_your_first_main(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    triggers::this_enters(q, asker.source, event).or_else(|| triggers::your_first_main_phase(q, asker, event))
}

/// The ability's two events as one trigger: a zone change, or a step beginning.
pub const ENTERS_OR_YOUR_FIRST_MAIN: Trigger =
    when_asked(EventMask(EventMask::ZONE_CHANGE.0 | EventMask::STEP_BEGAN.0), enters_or_your_first_main);

/// "The next artifact creature spell you cast this turn" (CR 611.2a): only that spell uses it up.
fn artifact_creature_spell(q: &dyn Query, spell: ObjectId) -> bool {
    let chars = q.chars(spell);
    chars.is(CardType::Artifact) && chars.is(CardType::Creature)
}

/// "The next Spacecraft spell you cast this turn": the subtype off the spell's own characteristics (CR 612.2).
fn spacecraft_spell(q: &dyn Query, spell: ObjectId) -> bool {
    q.chars(spell).has_subtype(Subtype::Spacecraft)
}

/// "...the next artifact creature spell and the next Spacecraft spell you cast this turn have undaunted": two lasting
/// effects, one per kind, each used up by the first spell of its kind and gone at end of turn (CR 611.2a, 514.2).
fn grant_undaunted(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.one_instruction(&mut |g| {
        g.spells_cost_this_turn(item, item.controller, 0);
        g.spells_cost_this_turn(item, item.controller, 1);
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        ENTERS_OR_YOUR_FIRST_MAIN,
        grant_undaunted,
        "When Võx enters and at the beginning of your first main phase, the next artifact creature spell and the next Spacecraft spell you cast this turn have undaunted. (They cost {1} less to cast for each opponent.)",
    )],
    lasting_cost_changes: &[
        LastingCostChange {
            text: "The next artifact creature spell you cast this turn has undaunted.",
            change: undaunted,
            next: Some(artifact_creature_spell),
        },
        LastingCostChange {
            text: "The next Spacecraft spell you cast this turn has undaunted.",
            change: undaunted,
            next: Some(spacecraft_spell),
        },
    ],
    ..oracle::V_X_CHILD_OF_THE_COLLECTIVE
};
