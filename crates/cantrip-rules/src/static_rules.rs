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
        // A permanent that has lost all its abilities has none of these rules (CR 613.1f).
        (has(def) && !cx.s().lost_all_abilities(id)).then_some((id, obj.controller, def))
    })
}

/// The kinds of zone `player` can't cast spells from right now (CR 101.2, 601.3): those a static ability of a permanent on the
/// battlefield forbids (Voice of Victory forbids them all on its controller's turn, Drannith Magistrate every zone but the
/// hand), and those an effect about the player does (`EffectKind::CantCastSpellsFrom`, Avatar's Wrath). A restriction takes
/// precedence over any permission to cast (CR 101.2).
pub fn cast_barred(cx: &Cx, player: PlayerId) -> ZoneKinds {
    let mut barred = ZoneKinds::NONE;
    for e in &cx.s().effects {
        if let EffectKind::CantCastSpellsFrom(p, zones) = e.kind
            && p == player
        {
            barred = barred.union(zones);
        }
    }
    for (id, controller, def) in sources(cx, |d| !d.cast_rules.is_empty()) {
        let asker = Asker { source: id, controller };
        for from in ZoneKinds::CAST_FROM {
            if !barred.contains(from) && def.cast_rules.iter().any(|rule| !rule(cx, &asker, player, from)) {
                barred = barred.with(from);
            }
        }
    }
    barred
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

/// Which of the `damaged` permanents keep their damage through this cleanup step (CR 514.2): those a static ability of a
/// permanent on the battlefield says so of (`CardDef::damage_stays`), while that permanent has its abilities (CR 613.1f).
/// Asked only in a cleanup step with damage marked, so the battlefield is searched then.
pub fn damage_stays(cx: &Cx, damaged: &[ObjectId]) -> Vec<ObjectId> {
    let rules: Vec<(ObjectId, PlayerId, &'static CardDef)> = sources(cx, |d| !d.damage_stays.is_empty()).collect();
    if rules.is_empty() {
        return Vec::new();
    }
    damaged
        .iter()
        .copied()
        .filter(|&object| {
            rules.iter().any(|&(id, controller, def)| {
                let asker = Asker { source: id, controller };
                def.damage_stays.iter().any(|rule| rule(cx, &asker, object))
            })
        })
        .collect()
}

/// Whether `player` can't gain life (CR 119.7): a resolved effect about them says so (`EffectKind::CantGainLife`, Screaming
/// Nemesis's "for the rest of the game"), or a static ability of a permanent on the battlefield does (`CardDef::cant_gain_life`,
/// Sunspine Lynx's "Players can't gain life"), which stops when the permanent leaves or loses its abilities (CR 611.3a,
/// 613.1f). Asked only when a player would gain life, so the battlefield is searched then.
pub fn cant_gain_life(q: &dyn Query, cards: &'static CardTable, player: PlayerId) -> bool {
    let s = q.state();
    if s.effects.iter().any(|e| e.kind == EffectKind::CantGainLife(player)) {
        return true;
    }
    s.battlefield().iter().any(|&id| {
        let obj = s.obj(id);
        let def = cards.get(cards.current(obj));
        !def.cant_gain_life.is_empty() && !s.lost_all_abilities(id) && {
            let asker = Asker { source: id, controller: obj.controller };
            def.cant_gain_life.iter().any(|rule| rule(q, &asker, player))
        }
    })
}

/// What each attack open to the active player's creatures costs to declare (CR 508.1d, 508.1h): for each of `eligible`
/// and each of `targets`, every static ability of a permanent on the battlefield that makes that attack cost something
/// (`CardDef::attack_costs`: Propaganda's "creatures can't attack you unless their controller pays {2} for each creature
/// they control that's attacking you"), added up, with the first such permanent named as the cost's source. Attacks that
/// cost nothing have no entry. A board with no such permanent, nearly every board, makes no list.
pub fn attack_costs(cx: &Cx, eligible: &[ObjectId], targets: &[Target]) -> Vec<AttackCost> {
    let taxes: Vec<(ObjectId, PlayerId, &'static CardDef)> = sources(cx, |d| !d.attack_costs.is_empty()).collect();
    if taxes.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for &attacker in eligible {
        for &target in targets {
            let mut total = ManaCost::ZERO;
            let mut first = None;
            for &(id, controller, def) in &taxes {
                let asker = Asker { source: id, controller };
                for rule in def.attack_costs {
                    if let Some(cost) = rule(cx, &asker, attacker, target)
                        && !cost.is_zero()
                    {
                        total = total.plus(&cost);
                        first.get_or_insert(id);
                    }
                }
            }
            if let Some(source) = first {
                out.push(AttackCost { attacker, target, cost: total, source });
            }
        }
    }
    out
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

/// Whether an effect lets `player` activate the loyalty abilities of `planeswalker`, a permanent they control, any time they
/// could cast an instant (`EffectKind::InstantLoyalty`, CR 606.3, 611.2c). Read from the effects in force, so a game with
/// none pays one pass over them, and only when a loyalty ability is asked about outside sorcery timing.
pub fn instant_loyalty(cx: &Cx, player: PlayerId, planeswalker: ObjectId) -> bool {
    cx.s().effects.iter().any(|e| match e.kind {
        EffectKind::InstantLoyalty(kind) if e.controller == player => {
            let c = cx.chars(planeswalker);
            c.is(CardType::Planeswalker) && kind.is_none_or(|t| c.has_subtype(t))
        }
        _ => false,
    })
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
            cx.cards().activation_rules[cx.cards().current(obj).0 as usize] && !cx.s().lost_all_abilities(id)
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
