use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature with the same name as this creature" (CR 115.1, 201.2): judged by the names both have now - a copy has
/// the name it copies - so the creature this one copied qualifies, and so does this creature itself.
fn creature_with_the_same_name(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.chars(id).is(CardType::Creature)
        && cantrip_mechanics::same_name(q, id, asker.source))
}

/// "Destroy target creature with the same name as this creature." Nothing if the target is no longer legal as the ability
/// resolves (CR 608.2b: the core doesn't resolve it).
fn destroy_the_same_name(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.destroy(creature);
    }
    Ok(())
}

/// The ability the copy's exception adds (CR 707.9b). Not a card anyone plays: a definition for the exception's ability to
/// be referred to by. Its {T} is a creature's, so it can't be paid the turn the creature came under its controller's
/// control unless it has haste (CR 302.6).
pub static EXCEPTION: CardDef = CardDef {
    name: "Evil Twin (copy exception)",
    abilities: &[ability(cost("{U}{B}, {T}"), destroy_the_same_name, "{U}{B}, {T}: Destroy target creature with the same name as this creature.")
        .targets(&[target(creature_with_the_same_name)])],
    ..CardDef::BLANK
};

/// "You may have this creature enter as a copy of any creature on the battlefield, except it has [the exception's ability]."
/// A choice, not a target (CR 608.2d): with one creature out it is taken without asking, and with none nothing is asked.
/// The ability is part of the copy's copiable values (CR 707.9b; the ruling), so a later copy of this creature has it too.
/// Declined, it enters as itself, a 0/0 with no abilities.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |q, copy| {
        copy.with_ability(AbilityRef { card: q.card_id(&EXCEPTION), index: 0 })
    })
}

pub static CARD: CardDef = CardDef { enters: Some(enter_as_copy), ..oracle::EVIL_TWIN };

pub static EXTRA: &[&CardDef] = &[&EXCEPTION];
