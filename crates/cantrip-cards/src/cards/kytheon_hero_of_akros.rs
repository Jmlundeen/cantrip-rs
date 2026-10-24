use cantrip_api::*;
use cantrip_mechanics::{effects, exile_and_return_transformed, history, prevention, targets, triggers};
use cantrip_oracle as oracle;

// --- Kytheon, Hero of Akros ----------------------------------------------------------------------------------------

/// "If Kytheon and at least two other creatures attacked this combat" (CR 603.4, 508.1): declared as attackers in this
/// combat phase, as the history has it - those that have left since still count, and one put onto the battlefield
/// attacking was never declared (the rulings). Asked as the ability triggers and again as it resolves.
fn he_and_two_others_attacked(q: &dyn Query, asker: &Asker) -> bool {
    let mut him = false;
    let mut others: Vec<ObjectId> = Vec::new();
    for r in history::this_combat(q) {
        if let Event::Attacks { attacker, .. } = r.event {
            if attacker == asker.source {
                him = true;
            } else if !others.contains(&attacker) {
                others.push(attacker);
            }
        }
    }
    him && others.len() >= 2
}

/// "Exile Kytheon, then return him to the battlefield transformed under his owner's control" (CR 712.14a): nothing if he
/// has left the battlefield already (CR 400.7).
fn transform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let owner = g.obj(item.object).owner;
    exile_and_return_transformed(g, item.object, owner)
}

// --- Gideon, Battle-Forged -----------------------------------------------------------------------------------------

/// "Up to one target creature an opponent controls attacks Gideon during its controller's next turn if able" (CR 508.1d,
/// 611.2a): a requirement on that creature during the next turn of whoever controls it then, to attack this Gideon. If he
/// can't be attacked then - gone, or no longer a planeswalker - nothing is required of it (the rulings).
fn plus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::MustAttackAt(Target::Object(item.object)), Duration::DURING_ITS_CONTROLLERS_NEXT_TURN);
    Ok(())
}

/// "Until your next turn, target creature gains indestructible. Untap that creature." (CR 611.2a, 701.26b): two sentences,
/// both of the target, if it is still legal.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::UntilNextTurnOf(item.controller));
    g.end_sentence();
    g.untap(creature);
    Ok(())
}

/// "Until end of turn, Gideon becomes a 4/4 Human Soldier creature with indestructible that's still a planeswalker.
/// Prevent all damage that would be dealt to him this turn."
fn zero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let keywords = Keywords::of(&[Keyword::Indestructible]);
    effects::planeswalker_becomes_creature_preventing_damage(g, item, (4, 4), None, &[Subtype::Human, Subtype::Soldier], keywords, 0);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        ability(loyalty(2), plus_two, "+2: Up to one target creature an opponent controls attacks Gideon during its controller's next turn if able.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS.optional()]),
        ability(loyalty(1), plus_one, "+1: Until your next turn, target creature gains indestructible. Untap that creature.").targets(&[targets::CREATURE]),
        ability(loyalty(0), zero, "0: Until end of turn, Gideon becomes a 4/4 Human Soldier creature with indestructible that's still a planeswalker. Prevent all damage that would be dealt to him this turn."),
    ],
    granted_preventions: &[prevention::ANY_DAMAGE],
    ..oracle::GIDEON_BATTLE_FORGED
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::END_OF_COMBAT, transform, "At end of combat, if Kytheon and at least two other creatures attacked this combat, exile Kytheon, then return him to the battlefield transformed under his owner's control.")
            .condition(he_and_two_others_attacked),
        ability(cost("{2}{W}"), effects::this_gains_indestructible, "{2}{W}: Kytheon gains indestructible until end of turn."),
    ],
    back: Some(&BACK),
    ..oracle::KYTHEON_HERO_OF_AKROS
};
