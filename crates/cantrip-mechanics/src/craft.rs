//! Craft with [materials] [cost] (CR 702.167a): "[Cost], Exile this permanent, Exile [materials] from among permanents
//! you control and/or cards in your graveyard: Return this card to the battlefield transformed under its owner's
//! control. Activate only as a sorcery." Dire Flail, Oteclan Landmark, Tithing Blade and the rest of The Lost Caverns
//! of Ixalan's double-faced artifacts print it.
//!
//! A card with craft lists `ability(cost, materials, text)` among its `abilities`, its materials one of the constants
//! here or `materials(..)` for its own. The exile of the source and of the materials is the core's (`Cost::materials`):
//! chosen with the rest of the cost and paid together, so a craft ability on the stack has nothing left to exile, and a
//! source gone in response is impossible - it went as the cost was paid.

use cantrip_api::*;

/// "Return this card to the battlefield transformed under its owner's control" (CR 702.167a, 712.14a): the card the
/// cost exiled, found where the cost put it (CR 400.7). Nothing if it has left exile since, or was a token, which
/// ceased to exist there (CR 111.7, 704.5d); a card that is not a double-faced card stays in exile (CR 712.14a), as one
/// that was only a copy of a craft card does (the craft rulings, 2023-11-10). Before it moves, the cards exiled as the
/// materials are linked to it in exile, so the permanent it becomes has them from the moment it enters (CR 702.167c,
/// `exiled_cards_used_to_craft`).
fn return_transformed(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    if !g.obj(card).zone.is(ZoneKind::Exile) || g.obj(card).token {
        return Ok(());
    }
    let materials = exiled_materials(g, item.object, item.ability);
    g.link_exiled(&materials, card);
    let owner = g.obj(card).owner;
    g.put_onto_battlefield_transformed(card, owner)?;
    Ok(())
}

/// The cards the cost of `source`'s craft `ability` exiled as its materials (CR 702.167a), still in exile: asked of the
/// history. The activation is recorded right after its cost was paid, and the core exiles a craft ability's source and
/// then its materials together as the last of the cost (`Cost::materials`), so the exiles just before the activation,
/// back to the source's own, are the materials. A token material ceased to exist (CR 111.7) and is none of them.
fn exiled_materials(q: &dyn Query, source: ObjectId, ability: AbilityRef) -> Vec<ObjectId> {
    let records = q.events();
    let activated = records
        .iter()
        .rposition(|r| matches!(r.event, Event::AbilityActivated { source: s, ability: a, .. } if s == source && a == ability));
    let Some(activated) = activated else { return Vec::new() };
    let mut out = Vec::new();
    for r in records[..activated].iter().rev() {
        match r.event {
            Event::ZoneChange { old, .. } if old == source => break,
            Event::ZoneChange { new, to, reason: ZoneReason::Exile, .. } if to.is(ZoneKind::Exile) => {
                if is_live(q, new) && q.obj(new).zone.is(ZoneKind::Exile) && !q.obj(new).token {
                    out.push(new);
                }
            }
            _ => {}
        }
    }
    out.reverse();
    out
}

/// Craft with `materials` `mana` (CR 702.167a): the mana, the exile of this permanent and of the materials, at sorcery
/// speed. `text` is the printed line, "Craft with artifact {3}{R}{R}".
pub const fn ability(mana: ManaCost, materials: &'static Materials, text: &'static str) -> AbilityDef {
    let cost = Cost { materials: Some(materials), ..Cost::mana(mana).and(EXILE_THIS) };
    cantrip_api::ability(cost, return_transformed, text).sorcery_speed()
}

/// Materials of a kind of one's own: `count` of them, or `count` or more, each passing `filter` (CR 702.167a-b).
pub const fn materials(count: u8, or_more: bool, filter: ObjectFilter, quality: &'static str) -> Materials {
    Materials { count, or_more, filter, quality }
}

/// A material named "artifact": a permanent of that type or a card of that type in the graveyard (CR 702.167b). The
/// candidates the core offers are only ever those, so the filter tests the type and nothing else; `materials(..)` takes it
/// for a count of a card's own ("six artifacts", Tetzin, Gnome Champion).
pub fn artifact(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Artifact)
}

/// A material named "creature", as `artifact` is.
pub fn creature(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature)
}

/// "Craft with artifact": another artifact you control or an artifact card from your graveyard (CR 702.167b). Dire Flail,
/// Oteclan Landmark, Idol of the Deep King, Inverted Iceberg, Lodestone Needle, Spring-Loaded Sawblades and more.
pub const ARTIFACT: Materials = materials(1, false, artifact, "artifact");

/// "Craft with creature": a creature you control or a creature card from your graveyard (CR 702.167b). Tithing Blade,
/// Jade Seedstones.
pub const CREATURE: Materials = materials(1, false, creature, "creature");

/// "Craft with two creatures": two from among creatures you control and creature cards in your graveyard. Visage of
/// Dread.
pub const TWO_CREATURES: Materials = materials(2, false, creature, "creature");

/// "Craft with one or more creatures": Paleontologist's Pick-Axe, Altar of the Wretched.
pub const ONE_OR_MORE_CREATURES: Materials = materials(1, true, creature, "creature");

/// The exiled cards used to craft `permanent` (CR 702.167c): the cards exiled to pay the activation cost of the craft
/// ability that put it onto the battlefield, as long as they remain in exile (the craft rulings, 2023-11-10), whoever
/// controls it now. A token exiled that way is no card and ceased to exist, so it is none of them; a permanent that got
/// onto the battlefield any other way, or left and came back, has none (CR 400.7). Asked of a permanent that has left (its
/// enters trigger resolving after it is gone), they are the ones it had as it last existed (CR 608.2h). In exile order.
/// Jade Seedstones's Jadeheart Attendant and Saheeli's Lattice's Mastercraft Raptor read them.
pub fn exiled_cards_used_to_craft(q: &dyn Query, permanent: ObjectId) -> Vec<ObjectId> {
    used_to_craft(q.state(), permanent)
}

/// `exiled_cards_used_to_craft` from the state alone, for a characteristic-defining ability (CR 604.3), which reads the game
/// through the layers: the materials are linked to the craft card as it waited in exile (`Game::link_exiled`), and that card
/// became `permanent` as it returned.
pub fn used_to_craft(s: &GameState, permanent: ObjectId) -> Vec<ObjectId> {
    s.seats()
        .flat_map(|p| s.zone(Zone::exile(p)).iter().copied())
        .filter(|&id| {
            let o = s.obj(id);
            o.became.is_none() && !o.token && o.exiled.is_some_and(|link| s.obj(link.by).became == Some(permanent))
        })
        .collect()
}
