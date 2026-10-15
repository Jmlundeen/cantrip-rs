//! Static abilities that change what may be cast or activated and what it costs (CR 601.2f, 602.2b, 101.2, 606.3). They
//! are found by looking at what is on the battlefield, and at nothing else while the battlefield has none, so a game
//! without them pays one table lookup per permanent when something is cast or activated.

use cantrip_api::*;

use crate::Cx;

/// The permanents with a static ability of this kind, with the player who controls each.
fn sources<'a>(cx: &'a Cx, has: impl Fn(&CardDef) -> bool + 'a) -> impl Iterator<Item = (ObjectId, PlayerId, &'static CardDef)> + 'a {
    cx.s().battlefield().iter().filter_map(move |&id| {
        let obj = cx.s().obj(id);
        let def = cx.card(cx.cards().current(obj));
        has(def).then_some((id, obj.controller, def))
    })
}

/// Whether anything on the battlefield forbids `player` to cast spells right now (CR 101.2).
pub fn may_cast(cx: &Cx, player: PlayerId) -> bool {
    for (id, controller, def) in sources(cx, |d| !d.cast_rules.is_empty()) {
        let asker = Asker { source: id, controller };
        if def.cast_rules.iter().any(|rule| !rule(cx, &asker, player)) {
            return false;
        }
    }
    true
}

/// Whether a static ability makes the damage `source` is about to deal unpreventable (CR 615.12). Asked only when
/// protection would prevent it, which is rare, so the battlefield is searched then.
pub fn damage_cant_be_prevented(cx: &Cx, source: ObjectId, combat: bool) -> bool {
    for (id, controller, def) in sources(cx, |d| !d.unpreventable_damage.is_empty()) {
        let asker = Asker { source: id, controller };
        if def.unpreventable_damage.iter().any(|rule| rule(cx, &asker, source, combat)) {
            return true;
        }
    }
    false
}

/// Whether `player` may spend mana as though it were of any color to activate an ability of `source` (CR 609.4b).
pub fn spend_any_color_for(cx: &Cx, player: PlayerId, source: ObjectId) -> bool {
    sources(cx, |d| !d.spend_any_color.is_empty()).any(|(id, controller, def)| {
        controller == player && def.spend_any_color.iter().any(|rule| rule(cx, &Asker { source: id, controller }, source))
    })
}

/// What `creature` assigns as combat damage (CR 510.1a): its power, or its toughness if a static ability says so.
pub fn combat_damage_of(cx: &Cx, creature: ObjectId) -> u16 {
    let c = cx.chars(creature);
    let by_toughness = sources(cx, |d| !d.damage_by_toughness.is_empty()).any(|(id, controller, def)| {
        let asker = Asker { source: id, controller };
        def.damage_by_toughness.iter().any(|rule| rule(cx, &asker, creature))
    });
    (if by_toughness { c.toughness } else { c.power }).unwrap_or(0).max(0) as u16
}

/// The card types `player` may cast as though they had flash (CR 307.5, 702.8): from effects that gave them that.
pub fn flash_types(cx: &Cx, player: PlayerId) -> TypeSet {
    let mut types = TypeSet::NONE;
    for e in &cx.s().effects {
        if let EffectKind::CastAsThoughFlash(t) = e.kind
            && e.controller == player
        {
            types = TypeSet(types.0 | t.0);
        }
    }
    types
}

/// The permanents whose static abilities let their controller cast some spells as though they had flash (CR 307.5): found
/// from the battlefield, and empty (so unallocated) in nearly every game. Asked once per question, then `grants_flash` per card.
pub fn flash_granters(cx: &Cx) -> Vec<(ObjectId, PlayerId, &'static CardDef)> {
    sources(cx, |d| !d.flash_rules.is_empty()).collect()
}

/// Whether one of the `granters` (from `flash_granters`) lets `player` cast `spell`, a card in the zone it is cast from, as
/// though it had flash (CR 307.5, 702.8).
pub fn grants_flash(cx: &Cx, granters: &[(ObjectId, PlayerId, &'static CardDef)], player: PlayerId, spell: ObjectId) -> bool {
    granters.iter().any(|&(id, controller, def)| {
        let asker = Asker { source: id, controller };
        def.flash_rules.iter().any(|rule| rule(cx, &asker, spell, player))
    })
}

/// The cost a static ability lets `player` pay rather than the mana cost of this ability of `permanent` (CR 118.9, 602.2b),
/// with the permanent that offers it.
pub fn activation_alternative(cx: &Cx, player: PlayerId, permanent: ObjectId, def: &ActivatedDef) -> Option<(ObjectId, ManaCost)> {
    for (id, controller, card) in sources(cx, |d| !d.ability_alternatives.is_empty()) {
        let asker = Asker { source: id, controller };
        for rule in card.ability_alternatives {
            if let Some(cost) = rule(cx, &asker, player, permanent, def) {
                return Some((id, cost));
            }
        }
    }
    None
}

/// Whether a static ability lets `player` activate this sorcery-speed ability whenever they could cast an instant.
pub fn instant_activation(cx: &Cx, player: PlayerId, source: ObjectId, index: u8) -> bool {
    for (id, controller, def) in sources(cx, |d| !d.instant_activation.is_empty()) {
        let asker = Asker { source: id, controller };
        if def.instant_activation.iter().any(|rule| rule(cx, &asker, source, index, player)) {
            return true;
        }
    }
    false
}

/// The permanents whose cards forbid activating abilities (CR 602.5): found from the board, not the card table, and empty
/// (so unallocated) in nearly every game. Asked once per question, then `activation_barred` per ability.
pub fn activation_forbidders(cx: &Cx) -> Vec<ObjectId> {
    cx.s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            cx.cards().activation_rules[cx.cards().current(obj).0 as usize]
        })
        .collect()
}

/// Whether one of the `forbidders` (from `activation_forbidders`) stops `source`'s ability `def` being activated (CR 602.5).
pub fn activation_barred(cx: &Cx, forbidders: &[ObjectId], source: ObjectId, def: &ActivatedDef) -> bool {
    forbidders.iter().any(|&id| {
        let obj = cx.s().obj(id);
        let asker = Asker { source: id, controller: obj.controller };
        cx.card(cx.cards().current(obj)).activation_rules.iter().any(|rule| !rule(cx, &asker, source, def))
    })
}
