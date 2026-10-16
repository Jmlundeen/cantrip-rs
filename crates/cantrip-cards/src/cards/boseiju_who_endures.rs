use cantrip_api::*;
use cantrip_mechanics::{channel, targets, effects, library::{search_to_battlefield, Find}};
use cantrip_oracle as oracle;

/// "Target artifact, enchantment, or nonbasic land an opponent controls" (CR 115.1).
fn opponents_target(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.controller(id) != asker.controller && {
            let c = q.chars(id);
            c.is(CardType::Artifact) || c.is(CardType::Enchantment) || (c.is(CardType::Land) && !c.supertypes.basic)
        })
}

/// Channel: destroy the target. Its controller (as it last was, CR 608.2h) may search their library for a land card with
/// a basic land type, put it onto the battlefield, then shuffle. The "may" is asked of that player, and declining skips
/// the search and the shuffle. Nothing happens at all if the target is no longer legal.
fn destroy_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let player = g.controller(target);
    g.destroy(target);
    if !effects::ask_may(g, player, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_battlefield(g, player, item.object, Find::ONE, false, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land)
            && [Subtype::Plains, Subtype::Island, Subtype::Swamp, Subtype::Mountain, Subtype::Forest]
                .into_iter()
                .any(|t| chars.has_subtype(t))
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(cost("{1}{G}").and(DISCARD_THIS), destroy_and_search, "Channel \u{2014} {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic land an opponent controls. That player may search their library for a land card with a basic land type, put it onto the battlefield, then shuffle. This ability costs {1} less to activate for each legendary creature you control.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: opponents_target, label: "" }])
            .from(ZoneKind::Hand),
    ],
    own_cost_changes: &[channel::cheaper_per_legendary_creature],
    ..oracle::BOSEIJU_WHO_ENDURES
};
