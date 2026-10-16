//! Trigger conditions and abilities many cards print.

use cantrip_api::*;

/// "When this enters" (CR 603.6a): the entering object's own zone change onto the battlefield.
pub fn this_enters(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Battlefield) => Fired::PLAIN,
        _ => None,
    }
}

/// "When this enters, if you cast it" (CR 603.4, 603.6a): the permanent entered as the spell that resolved, and that spell
/// was cast - the history says so. A permanent put onto the battlefield, or a copy of a permanent spell (a token, CR 707.10f),
/// was not cast. Lutri, The One Ring and Uldaros Theorix print it.
pub fn this_enters_if_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, to, reason: ZoneReason::Resolve, .. }
            if *new == source && to.is(ZoneKind::Battlefield) && q.history().cast_of(*old).is_some() =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// An intervening "if it's on the battlefield" (CR 603.4), checked as the ability triggers and again as it resolves: the
/// permanent is still the object that triggered (CR 400.7) and has not left. Animate Dead and Necromancy print it.
pub fn this_is_on_the_battlefield(q: &dyn Query, asker: &Asker) -> bool {
    q.is_on_battlefield(asker.source) && q.obj(asker.source).became.is_none()
}

/// Landfall (CR 207.2c): a land entering the battlefield under your control, by any means. Icetill Explorer and
/// Tireless Tracker print it.
pub fn landfall(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Land) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "When you unlock this door" on the Room's right half (CR 709.5h): the Room permanent was given that half's unlocked
/// designation by its controller, whether as it entered because the half was cast (CR 709.5d) or after (CR 709.5e-f).
/// Ritual Chamber, Awakening Hall and Cadaver Lab print it. (The left half's is the same with `DOOR_LEFT`, added with the
/// first card that has one.)
pub fn you_unlock_this_right_door(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    you_unlock_this_door(q, source, event, DOOR_RIGHT)
}

fn you_unlock_this_door(q: &dyn Query, source: ObjectId, event: &Event, door: u8) -> Option<Fired> {
    match event {
        Event::DoorUnlocked { permanent, door: unlocked, player } if *permanent == source && *unlocked == door && *player == q.controller(source) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "When you cast this spell" (CR 603.2, 601.2i): the spell itself, from wherever it was cast. Sage of the Skies and
/// every cascade card print it.
pub fn cast_this_spell(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if *spell == source => Fired::PLAIN,
        _ => None,
    }
}

/// Cascade's effect (CR 702.85a): exile cards from the top of the library until a nonland card with a lesser mana
/// value than the spell's; its controller may cast it without paying its mana cost; then every card exiled and not
/// cast goes to the bottom of the library in a random order. The spell's mana value is its card's, X included, as it
/// is now or as it last was (CR 608.2h). Judged by the card rather than the face cast (an Adventure's cheaper face is
/// not considered).
fn cascade_resolves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let def = g.card(g.chars(item.object).card);
    let limit = def.cost.mana_value() + def.cost.x as u32 * g.obj(item.object).x as u32;

    let mut exiled: Vec<ObjectId> = Vec::new();
    let mut hit = None;
    while let Some(&top) = g.state().zone(Zone::library(player)).first() {
        let moved = g.exile(top);
        if !g.obj(moved).zone.is(ZoneKind::Exile) {
            continue;
        }
        exiled.push(moved);
        let chars = g.chars(moved);
        if !chars.is(CardType::Land) && g.card(chars.card).cost.mana_value() < limit {
            hit = Some(moved);
            break;
        }
    }
    // "You may cast it without paying its mana cost if the resulting spell's mana value is less than this spell's": the face
    // cast is the spell, so only a face under the limit may be (CR 702.85a, 709.3b, 715.3b).
    if let Some(card) = hit {
        let ways: Vec<CastWay> = g.cast_ways_during_resolution(card).into_iter().filter(|w| u32::from(w.mana_value) < limit).collect();
        if !ways.is_empty() && matches!(g.ask(player, RequestKind::May { source: card, purpose: Purpose::CastOrBottom })?, Answer::Yes)
            && let Some(way) = crate::choose::cast_way(g, player, card, ways)?
        {
            g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), way: Some(way.way), ..CastTerms::default() })?;
        }
    }

    // What was cast has left exile with its identity; the rest go to the bottom, shuffled among themselves.
    let rest: Vec<ObjectId> = exiled.into_iter().filter(|&c| is_live(g, c) && g.obj(c).zone.is(ZoneKind::Exile)).collect();
    g.put_into_library(&rest, LibraryPlace::BottomInRandomOrder);
    Ok(())
}

/// Annihilator N (CR 702.86a): "Whenever this creature attacks, defending player sacrifices N permanents." The defending
/// player is the one attacked, or the controller of the planeswalker or protector of the battle attacked (CR 508.5).
/// Emrakul, the Aeons Torn and Ulamog's Crusher print it.
pub const fn annihilator<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::ATTACKS, this_attacks_defender), annihilate::<N>, text)
}

/// "Whenever this creature attacks" (CR 508.1m, 603.2): its own attack, whatever it attacks. Aragorn, King of Gondor and
/// Caustic Bronco print it. The `listens` mask is `EventMask::ATTACKS`.
pub fn this_attacks(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever this creature attacks" where the ability is about whom it attacks: the player attacked, or the
/// planeswalker or battle (CR 508.5). The defender is the trigger's subject. Generous Plunderer and Otharri, Sun's
/// Glory print it.
pub fn this_attacks_defender(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender } if *attacker == source => Fired::about(*defender),
        _ => None,
    }
}

/// "Whenever equipped creature attacks" (CR 508.1m, 301.5): the creature this Equipment is attached to attacks. Adaptive
/// Omnitool and Argentum Armor print it. `equipped_creature_attacks_subject` carries the creature as the subject.
pub fn equipped_creature_attacks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.attached_to(source) == Some(*attacker) => Fired::PLAIN,
        _ => None,
    }
}

/// `equipped_creature_attacks` with the attacking creature as the trigger's subject, for an ability that does something
/// to it ("it gains ..."). Bone Sabres and Strength-Testing Hammer print it.
pub fn equipped_creature_attacks_subject(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.attached_to(source) == Some(*attacker) => Fired::about(Target::Object(*attacker)),
        _ => None,
    }
}

/// "Whenever enchanted player is attacked" (CR 508.1m, 303.4): a creature attacks the player this Curse is attached to,
/// directly. The attacking creature is the trigger's subject. Curse of Opulence and Curse of Predation print it.
pub fn enchanted_player_attacked(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender: Target::Player(p) } if q.obj(source).attached_player == Some(*p) => {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

/// "Whenever you attack" (CR 508.1m): the controller declares attackers, whatever their number. Guide of Souls and Inti,
/// Seneschal of the Sun print it.
pub fn you_attack(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::AttackersDeclared { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn annihilate<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(defending) = g.defending_player(item.subject) else { return Ok(()) };
    crate::sacrifice_chosen(g, defending, item.object, N, |_| true)
}

/// Cascade (CR 702.85a): "When you cast this spell, exile cards from the top of your library until you exile a nonland
/// card that costs less. You may cast it without paying its mana cost. Put all cards exiled this way that weren't cast
/// on the bottom of your library in a random order." Bloodbraid Challenger and Bloodbraid Elf print it.
pub const CASCADE: AbilityDef = triggered(when(EventMask::SPELL_CAST, cast_this_spell), cascade_resolves, "Cascade")
    .anywhere();

fn cycle_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// Cycling [cost] (CR 702.29a): "[cost], Discard this card: Draw a card", from the hand. `text` is the ability's
/// full text, which a scenario's `Ability:` matches.
pub const fn cycling(mana: ManaCost, text: &'static str) -> AbilityDef {
    ability(Cost::mana(mana).and(DISCARD_THIS), cycle_draw, text)
        .from(ZoneKind::Hand)
}

fn lone_attacker(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. }
            if q.state().attackers.len() == 1 && q.controller(*attacker) == q.controller(source) =>
        {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

fn exalted_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        crate::pump(g, item.object, creature, 1, 1);
    }
    Ok(())
}

/// Exalted (CR 702.83a): "Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn."
/// Ignoble Hierarch and Noble Hierarch print it word for word.
pub const EXALTED: AbilityDef = triggered(when(EventMask::ATTACKS, lone_attacker), exalted_pump, "Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.");

/// "When this creature enters, if its squad cost was paid": an unpaid cast, or a token copy, does not trigger at all.
fn entered_squadded(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Battlefield) && q.obj(source).kicked > 0 => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Create a token that's a copy of it for each time its squad cost was paid, all at once, from its copiable values
/// as the ability resolves.
fn squad_copies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let times = g.obj(item.object).kicked as u32;
    if times > 0 {
        g.create_token_copies(item.object, item.controller, times)?;
    }
    Ok(())
}

/// Squad (CR 702.157a): the enters trigger. Securitron Squadron and Sicarian Infiltrator print it word for word; the
/// additional cost is the card's `kicker`.
pub const SQUAD: AbilityDef = triggered(when(EventMask::ZONE_CHANGE, entered_squadded), squad_copies, "Squad: When this creature enters, if its squad cost was paid, create a token that's a copy of it for each time its squad cost was paid.");

fn ward_matches(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameTarget { target: Target::Object(t), by, controller } if *t == source && *controller != q.controller(source) => {
            // What targeted it, who controls that, and where it is on the stack (it is the top item as it is announced).
            Some(Fired {
                subject: Target::Object(*by),
                data: [controller.0 as u32, q.state().stack.len().saturating_sub(1) as u32],
            })
        }
        _ => None,
    }
}

fn ward_effect<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(by) = item.subject else { return Ok(()) };
    let payer = PlayerId(item.data[0] as u8);
    if !g.unless_pays(payer, item.object, by, generic(N))? {
        g.counter_item(by, item.data[1] as usize);
    }
    Ok(())
}

/// Ward {N} (CR 702.21a): "Whenever this becomes the target of a spell or ability an opponent controls, counter it
/// unless that player pays {N}." A creature granted ward (Lavaspur Boots) has it from the granting card's delayed
/// abilities; Kappa Cannoneer prints it with {4}.
pub const fn ward<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::BECAME_TARGET, ward_matches), ward_effect::<N>, text)
        .ward()
}

fn ward_life_effect<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(by) = item.subject else { return Ok(()) };
    let payer = PlayerId(item.data[0] as u8);
    // A cost of life is paid only by a player with that much (CR 119.4), so nobody else is asked; the payer says whether
    // to pay, as the C# engine's unlessPaysLife asked it.
    let paid = g.state().player(payer).life >= N as i32
        && g.ask(payer, RequestKind::May { source: item.object, purpose: Purpose::Optional })? == Answer::Yes;
    if paid {
        g.lose_life(payer, N as i32);
    } else {
        g.counter_item(by, item.data[1] as usize);
    }
    Ok(())
}

/// Ward--Pay N life (CR 702.21a): "Whenever this becomes the target of a spell or ability an opponent controls, counter it
/// unless that player pays N life." Sedgemoor Witch prints it with 3.
pub const fn ward_life<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::BECAME_TARGET, ward_matches), ward_life_effect::<N>, text)
        .ward()
}

fn ward_discard_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(by) = item.subject else { return Ok(()) };
    let payer = PlayerId(item.data[0] as u8);
    // A player with no card in hand cannot pay and is not asked; one who can is asked whether to, and then which card.
    if !g.state().zone(Zone::hand(payer)).is_empty()
        && matches!(g.ask(payer, RequestKind::May { source: by, purpose: Purpose::Discard })?, Answer::Yes)
    {
        crate::discard_chosen(g, payer, item.object, 1)?;
        return Ok(());
    }
    g.counter_item(by, item.data[1] as usize);
    Ok(())
}

/// Ward--Discard a card (CR 702.21a, 118.12): "Whenever this becomes the target of a spell or ability an opponent controls,
/// counter it unless that player discards a card." Sunset Saboteur prints it.
pub const fn ward_discard(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::BECAME_TARGET, ward_matches), ward_discard_effect, text)
        .ward()
}

/// "Whenever you cast a noncreature spell" (CR 603.2): prowess, Displacer Kitten and Dragon's Rage Channeler print it.
pub fn noncreature_spell_you_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && !q.chars(*spell).is(CardType::Creature) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn prowess_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    crate::pump(g, item.object, item.object, 1, 1);
    Ok(())
}

/// Prowess (CR 702.108a): "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn", as a
/// real triggered ability. The Otter token of Stormchaser's Talent and the Monk token of Cori-Steel Cutter carry it.
pub const PROWESS: AbilityDef = triggered(when(EventMask::SPELL_CAST, noncreature_spell_you_cast), prowess_pump, "Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)");

/// "When this is put into a graveyard from the battlefield" (CR 700.4 for a creature: "dies"), looked back on: the
/// ability is the object's as it last existed there (CR 603.10a). Use with `look_back: true`.
pub fn this_dies(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. } if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "When this leaves the battlefield" (CR 603.6c, 603.10a), looked back on whichever zone it goes to. Use with
/// `look_back: true`. Animate Dead and Cryogen Relic print it.
pub fn this_leaves(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. } if *old == source && from.is(ZoneKind::Battlefield) => Fired::PLAIN,
        _ => None,
    }
}

/// "At the beginning of your [step]" (CR 503.1a, 603.2): `step` began on its controller's turn.
fn your_step(q: &dyn Query, source: ObjectId, event: &Event, step: Step) -> Option<Fired> {
    match event {
        Event::StepBegan { step: began } if *began == step && q.state().turn.active == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "At the beginning of your upkeep" (CR 503.1a). Dark Confidant and Court of Garenbrig print it.
pub fn your_upkeep(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    your_step(q, source, event, Step::Upkeep)
}

/// "At the beginning of combat on your turn" (CR 507.1). Luminarch Aspirant and Ouroboroid print it.
pub fn beginning_of_your_combat(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    your_step(q, source, event, Step::BeginCombat)
}

/// "At the beginning of your end step" (CR 513.1). Dawn of a New Age and Feywild Caretaker print it.
pub fn your_end_step(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    your_step(q, source, event, Step::End)
}

/// "At the beginning of each end step" (CR 513.1, 603.2): any player's end step began. Gau, Feral Youth and Octomancer print it.
pub fn each_end_step(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::End } => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever a creature you control enters" (CR 603.6a): any creature, this one included, under your control as it
/// entered. Aura Shards and Warleader's Call print it.
pub fn creature_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever another creature you control enters" (CR 603.6a): a creature other than this permanent, under your control
/// as it entered. The entering creature is the trigger's subject. Aunt May and Guide of Souls print it.
pub fn another_creature_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Whenever a creature dies" (CR 700.4, 603.10a), looked back on: any creature, anyone's, as it last was on the battlefield.
/// The ability carries the player who then controlled it (CR 608.2h). Use with `look_back: true`. Fecundity and Ashes of the
/// Abhorrent print it.
pub fn creature_dies(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.chars(*old).is(CardType::Creature) =>
        {
            Fired::about(Target::Player(q.controller(*old)))
        }
        _ => None,
    }
}

/// "Whenever a creature you control dies" (CR 700.4, 603.10a), looked back on: the creature as it last was on the
/// battlefield, under the controller it last had. Use with `look_back: true`. Grave Venerations and Warhost's Frenzy
/// print it.
pub fn creature_you_control_dies(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.chars(*old).is(CardType::Creature)
                && q.obj(*old).controller == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever you cast an artifact spell" (CR 603.2). Pinnacle Emissary and Ravenous Robots print it.
pub fn artifact_spell_you_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && q.chars(*spell).is(CardType::Artifact) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// `deals_combat_damage_to_player` for an ability about the damaged player or the damage: the player is the trigger's
/// subject and the amount of damage its `data[0]`. Etali, Primal Conqueror and Fallen Shinobi print it.
pub fn deals_combat_damage_to_player_subject(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(p), amount, combat: true } if *from == source && *amount > 0 => {
            Some(Fired { subject: Target::Player(*p), data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "Whenever this creature deals combat damage to a player" (CR 510.2, 603.2): its own damage, greater than zero.
/// Bejeweled Warg prints it; Carnage, Crimson Chaos's granted ability has the same condition.
pub fn deals_combat_damage_to_player(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(_), amount, combat: true } if *from == source && *amount > 0 => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever you flip a coin" (CR 705.1): any flip its controller makes, won or lost.
pub fn you_flip_a_coin(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::CoinFlipped { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever you win a coin flip" (CR 705.2): a flip its controller wins.
pub fn you_win_a_flip(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::CoinFlipped { player, won: true } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever you roll a die" (CR 706.2): a die its controller rolls, whatever its sides.
pub fn you_roll_a_die(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::DieRolled { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

// --- The triggers, each with the events it listens to: `triggered(THIS_ENTERS, effect, text)` ---------------------

pub const THIS_ENTERS: Trigger = when(EventMask::ZONE_CHANGE, this_enters);
pub const THIS_ENTERS_IF_CAST: Trigger = when(EventMask::ZONE_CHANGE, this_enters_if_cast);
pub const LANDFALL: Trigger = when(EventMask::ZONE_CHANGE, landfall);
pub const YOU_UNLOCK_THIS_RIGHT_DOOR: Trigger = when(EventMask::DOOR_UNLOCKED, you_unlock_this_right_door);
pub const CAST_THIS_SPELL: Trigger = when(EventMask::SPELL_CAST, cast_this_spell);
pub const THIS_ATTACKS: Trigger = when(EventMask::ATTACKS, this_attacks);
pub const THIS_ATTACKS_DEFENDER: Trigger = when(EventMask::ATTACKS, this_attacks_defender);
pub const EQUIPPED_CREATURE_ATTACKS: Trigger = when(EventMask::ATTACKS, equipped_creature_attacks);
pub const EQUIPPED_CREATURE_ATTACKS_SUBJECT: Trigger = when(EventMask::ATTACKS, equipped_creature_attacks_subject);
pub const ENCHANTED_PLAYER_ATTACKED: Trigger = when(EventMask::ATTACKS, enchanted_player_attacked);
pub const YOU_ATTACK: Trigger = when(EventMask::ATTACKERS_DECLARED, you_attack);
pub const NONCREATURE_SPELL_YOU_CAST: Trigger = when(EventMask::SPELL_CAST, noncreature_spell_you_cast);
pub const THIS_DIES: Trigger = when(EventMask::ZONE_CHANGE, this_dies);
pub const THIS_LEAVES: Trigger = when(EventMask::ZONE_CHANGE, this_leaves);
pub const YOUR_UPKEEP: Trigger = when(EventMask::STEP_BEGAN, your_upkeep);
pub const BEGINNING_OF_YOUR_COMBAT: Trigger = when(EventMask::STEP_BEGAN, beginning_of_your_combat);
pub const YOUR_END_STEP: Trigger = when(EventMask::STEP_BEGAN, your_end_step);
pub const EACH_END_STEP: Trigger = when(EventMask::STEP_BEGAN, each_end_step);
pub const CREATURE_YOU_CONTROL_ENTERS: Trigger = when(EventMask::ZONE_CHANGE, creature_you_control_enters);
pub const CREATURE_DIES: Trigger = when(EventMask::ZONE_CHANGE, creature_dies);
pub const CREATURE_YOU_CONTROL_DIES: Trigger = when(EventMask::ZONE_CHANGE, creature_you_control_dies);
pub const ARTIFACT_SPELL_YOU_CAST: Trigger = when(EventMask::SPELL_CAST, artifact_spell_you_cast);
pub const DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT: Trigger = when(EventMask::DAMAGE, deals_combat_damage_to_player_subject);
pub const DEALS_COMBAT_DAMAGE_TO_PLAYER: Trigger = when(EventMask::DAMAGE, deals_combat_damage_to_player);
