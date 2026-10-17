//! Pieces of static abilities (CR 604, 613) several printed cards use exactly as written.

use cantrip_api::*;

/// "Creatures you control" (CR 613.1f, layer 6): every creature under the source's controller's control, the source itself
/// among them ("creatures you control have ..." is not "other"). Judged on what the earlier layers made the object, so a
/// permanent that became a creature counts, and by control, not ownership. Nadu, Winged Wisdom and Samut, Hazoret's Champion
/// use it as `StaticDef::affects`.
pub fn your_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    so_far.is(CardType::Creature) && s.obj(affected).controller == s.obj(source).controller
}

/// "Planeswalkers you control" (CR 613.1f, layer 6): every planeswalker under the source's controller's control, judged on what
/// the earlier layers made it, and by control, not ownership. Avatar of Burgeoning Echoes and Way of the Pyromancer use it as
/// `StaticDef::affects`.
pub fn your_planeswalkers(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    so_far.is(CardType::Planeswalker) && s.obj(affected).controller == s.obj(source).controller
}

/// "Lands you control" (CR 613.1f, layer 6): every land under the source's controller's control, judged on what the earlier
/// layers made it, and by control, not ownership. The emblem of Nissa, Who Shakes the World and Resonating Lute use it as
/// `StaticDef::affects`.
pub fn your_lands(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    so_far.is(CardType::Land) && s.obj(affected).controller == s.obj(source).controller
}

/// "Creature tokens you control" (CR 111.1, 613.1f): a creature under the source's controller's control, as the earlier
/// layers made it, that is a token. Teysa Karlov, Caretaker's Talent (at level 3) and Gideon's Memorial use it as
/// `StaticDef::affects`.
pub fn your_creature_tokens(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    q.state().obj(affected).token && your_creatures(q, source, affected, so_far)
}

/// `your_spells_of_types::<ANY_SPELL>`: every spell, whatever its card types (Hexing Squelcher).
pub const ANY_SPELL: u16 = 0;

/// "[Creature / Instant and sorcery / No qualifier] spells you control" (CR 113.6, 601.2i, layer 6) as the `StaticDef::affects` of a
/// static ability declared `.everywhere()`: a spell on the stack (an ability on the stack has no card types) controlled by the
/// source's controller that has one of the card types in `TYPES`, a bitset of `CardType as u16` positions (`TypeSet.0`), judged on
/// what the earlier layers made it; `TYPES` of 0 is every spell. Frenzied Baloth (creature spells), Hexing Squelcher (spells) and
/// Sphinx of the Final Word (instant and sorcery spells) print it before "can't be countered".
pub fn your_spells_of_types<const TYPES: u16>(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let o = q.state().obj(affected);
    let of_a_type = if TYPES == 0 { so_far.types != TypeSet::NONE } else { so_far.types.0 & TYPES != 0 };
    o.zone.kind() == ZoneKind::Stack && of_a_type && o.controller == q.state().obj(source).controller
}

/// A static ability that affects only the object that has it (CR 604.1: "this creature", "this land", "this token") as its
/// `StaticDef::affects`. Anje's Ravager, Emrakul, the Aeons Torn, Multiversal Passage, Roaming Throne, Territorial Kavu and
/// the Construct token use it.
pub fn itself(_: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected
}

/// "Delirium - This creature gets/has ... as long as there are four or more card types among cards in your graveyard"
/// (CR 207.2c, 604.1, 611.3a) as a static ability's `StaticDef::affects`: only the source itself, and only while four or more
/// card types are among the cards in its controller's graveyard, each judged as the layers make it (the types are asked
/// again whenever characteristics are computed). Dragon's Rage Channeler and Wildfire Wickerfolk use it; a spell or an
/// ability asked at a `Query` uses `cantrip_mechanics::delirium`.
pub fn itself_with_delirium(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && delirium(q, source)
}

/// "... has haste" (CR 702.10, layer 6) as a static ability's `StaticDef::apply`. Anger, Samut, Lavaspur Boots, Skateboard
/// and dash's haste print it.
pub fn have_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Haste);
}

/// "... can't be countered" (CR 101.2, layer 6) as the `StaticDef::apply` of a static ability that covers spells on the
/// stack (declared `everywhere`). Hexing Squelcher ("Spells you control") and Frenzied Baloth ("Creature spells you
/// control") print it.
pub fn cant_be_countered(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::CantBeCountered);
}

/// "... has flying" (CR 702.9, layer 6) as a static ability's `StaticDef::apply`. Dragon's Rage Channeler's delirium and Iron
/// Man Armor's equipped creature print it.
pub fn have_flying(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying);
}

/// "... has deathtouch" (CR 702.2, layer 6) as a static ability's `StaticDef::apply`. Bow of Nylea's attacking creatures and
/// The Swarmweaver's delirium Insects and Spiders print it.
pub fn have_deathtouch(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Deathtouch);
}

/// Delirium (an ability word, CR 207.2c) for a static ability's condition: four or more card types among the cards in the
/// graveyard of `source`'s controller, judged on the cards' characteristics as the layers made them. `mechanics::delirium` is
/// the same question outside the layers. `itself_with_delirium` (Dragon's Rage Channeler, Wildfire Wickerfolk) and The
/// Swarmweaver's Insects and Spiders use it.
pub fn delirium(view: &dyn LayerQuery, source: ObjectId) -> bool {
    let state = view.state();
    let you = state.obj(source).controller;
    let mut seen = TypeSet::NONE;
    for &card in state.zone(Zone::graveyard(you)) {
        for t in view.chars(card).types.iter() {
            seen = seen.with(t);
        }
    }
    seen.iter().count() >= 4
}

/// "... has vigilance" (CR 702.20, layer 6) as a static ability's `StaticDef::apply`. Ori, Keeper of Songs, Gideon's Memorial's
/// creature tokens and Teysa Karlov's print it.
pub fn have_vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Vigilance);
}

/// "Each artifact you control" counted for a static ability (CR 613.4c): the permanents under the controller of `source` that
/// the earlier layers made artifacts, `source` itself among them if it is one. "+1/+1 for each artifact you control"
/// (below, Adaptive Omnitool and others) and Improvised Arsenal's "+1/+0 for each artifact you control" both count them.
pub fn artifacts_you_control(view: &dyn LayerQuery, source: ObjectId) -> i16 {
    let state = view.state();
    let you = state.obj(source).controller;
    state.battlefield().iter().filter(|&&id| state.obj(id).controller == you && view.chars(id).is(CardType::Artifact)).count() as i16
}

/// "Gets +1/+1 for each artifact you control" (CR 613.4c, layer 7c): "you" is the controller of the source, the object that has
/// the ability (CR 109.5, 113.6), counted as the earlier layers left every permanent, the source itself among them if it is an
/// artifact. The Construct token (Urza's Saga, Karn, Scion of Urza), Adaptive Omnitool's equipped creature and Iron Man
/// Armor's animated Equipment print it.
pub fn plus_one_plus_one_per_artifact_you_control(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = artifacts_you_control(view, source);
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

/// "... has trample" (CR 702.19, layer 6) as a static ability's `StaticDef::apply`. Ajani's Anguish, Keen-Eyed Curator, Orcrist
/// and Rancor print it.
pub fn have_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample);
}

/// "... has double strike and trample" (CR 702.4, 702.19, layer 6) as a static ability's `StaticDef::apply`. Embercleave and
/// Leyline Axe print it.
pub fn have_double_strike_and_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::DoubleStrike).with(Keyword::Trample);
}

/// "If this card is in your opening hand, you may begin the game with it on the battlefield" (CR 103.6a), for `CardDef::opening_hand`:
/// any player may, nothing else is required. The Leylines (Punishment and Resonance) and Leyline Axe print it.
pub const BEGIN_ON_THE_BATTLEFIELD: OpeningHandAction = OpeningHandAction { condition: in_any_opening_hand, counter: None, exile_from_hand: 0 };

fn in_any_opening_hand(_: &dyn Query, _: &Asker) -> bool {
    true
}

/// "This creature gets +1/+0 as long as any player controls a [color] permanent" (layer 7c) as a static ability's
/// `StaticDef::apply`, `COLOR` a `Color` as `u8`: no matter how many there are, and whoever controls them. The color is a
/// word of the source's text (CR 612.2), and a permanent's color is what the layers before this one made it. Knight of
/// Grace (black) and Knight of Malice (white) print it.
pub fn plus_one_zero_while_any_player_controls<const COLOR: u8>(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let color = view.words(source).color(Color::ALL[COLOR as usize]);
    let state = view.state();
    if state.battlefield().iter().any(|&id| view.chars(id).colors.has(color)) {
        chars.power = chars.power.map(|p| p + 1);
    }
}

/// The permanent an event put onto the battlefield (CR 603.6a): a zone change to the battlefield, a token's creation
/// included. What stops "[permanents] entering" from causing abilities to trigger asks it of the event, then judges the
/// permanent as it exists on the battlefield (`Query::chars` of the new object: copy effects and the other continuous effects
/// that apply to it, the rulings' March of the Machines). Torpor Orb, Doorkeeper Thrull and Hushbringer read it.
pub fn entering(event: &Event) -> Option<ObjectId> {
    match *event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield) => Some(new),
        _ => None,
    }
}

/// Whether an event is a creature dying that causes this ability to trigger (CR 700.4, 603.10a): a creature put into a
/// graveyard from the battlefield, judged as it last existed there (a land that had become a creature dies as a creature).
/// Two abilities the event matches are not caused by the dying (the rulings of Hushbringer, Teysa Karlov and Drivnod): one
/// whose trigger event is the sacrifice that caused it (CR 701.21a, "whenever you sacrifice a creature"), and the dying card's
/// own ability that triggers when it is put into a graveyard from anywhere, which functions from the graveyard it reached.
/// Hushbringer ("creatures ... dying don't cause abilities to trigger"), Teysa Karlov and Drivnod, Carnage Dominus ("if a
/// creature dying causes a triggered ability ... to trigger") ask it.
pub fn creature_dying(q: &dyn Query, event: &Event, triggering: &Triggering) -> bool {
    match *event {
        Event::ZoneChange { old, new, from, to, .. } if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => {
            q.chars(old).is(CardType::Creature) && !triggering.def.on_sacrifice && triggering.source != new
        }
        _ => false,
    }
}

/// "If a permanent entering causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time" (CR 603.2d, 603.6a) as a `CardDef::additional_triggers` entry: the permanent's own enters abilities and
/// every other ability its entering triggers, whoever controls the permanent (the rulings). Elesh Norn, Mother of Machines,
/// Yarok, the Desecrated, Starfield Vocalist and Virtue of Knowledge print it.
pub fn permanent_entering_triggers_again(_: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    entering(event).is_some() && triggering.of_a_permanent_controlled_by(asker.controller)
}

/// "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional
/// time" (CR 603.2d) as a `CardDef::additional_triggers` entry; what counts as the dying is [`creature_dying`]. Teysa Karlov
/// and Drivnod, Carnage Dominus print it.
pub fn creature_dying_triggers_again(q: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    triggering.of_a_permanent_controlled_by(asker.controller) && creature_dying(q, event, triggering)
}

/// "If a creature attacking causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time" (CR 603.2d, 508.2a) as a `CardDef::additional_triggers` entry: abilities whose trigger is a creature being
/// declared as an attacker, "whenever [this] attacks" and "whenever you attack" alike, not one that triggers on something else
/// attacking does, such as becoming tapped (the rulings). Isshin, Two Heavens as One and Windcrag Siege (Mardu) print it.
pub fn creature_attacking_triggers_again(_: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    matches!(event, Event::Attacks { .. } | Event::AttackersDeclared { .. }) && triggering.of_a_permanent_controlled_by(asker.controller)
}

/// "Creatures entering don't cause abilities to trigger" (CR 603.2, 603.6a) as a `CardDef::trigger_rules` entry: an
/// event that puts a creature onto the battlefield triggers no ability, the creature's own enters abilities included and
/// whoever controls them. Replacement effects and "as [this] enters" are not triggered abilities (CR 614.1c) and still
/// apply. Torpor Orb, Tocatli Honor Guard and Hushwing Gryff print it; Hushbringer prints it with "or dying".
pub fn creatures_entering_dont_trigger(q: &dyn Query, _: &Asker, event: &Event, _: &Triggering) -> bool {
    entering(event).is_some_and(|permanent| q.chars(permanent).is(CardType::Creature))
}

/// "Players can't gain life" (CR 119.7) as a `CardDef::cant_gain_life` entry: every player, its controller included, while
/// the permanent is on the battlefield. Sunspine Lynx, Giant Cindermaw, Rampaging Ferocidon, Leyline of Punishment, Everlasting
/// Torment and Havoc Festival print it.
pub fn players_cant_gain_life(_: &dyn Query, _: &Asker, _: PlayerId) -> bool {
    true
}

/// "Your opponents can't gain life" (CR 119.7, 102.2) as a `CardDef::cant_gain_life` entry: every player but its controller.
/// Knight of Dusk's Shadow, Erebos, God of the Dead, Gríma Wormtongue, Quakebringer and The Lord of Pain print it.
pub fn opponents_cant_gain_life(_: &dyn Query, asker: &Asker, player: PlayerId) -> bool {
    player != asker.controller
}

/// "Damage can't be prevented" (CR 615.12) as a `CardDef::unpreventable_damage` entry: all damage, from any source, combat
/// or not, while the permanent is on the battlefield. Sunspine Lynx, Leyline of Punishment and Everlasting Torment print it.
pub fn damage_cant_be_prevented(_: &dyn Query, _: &Asker, _: ObjectId, _: bool) -> bool {
    true
}

/// "Gets +P/+T" (CR 613.4c): the power and toughness change of an Equipment's, Aura's or permanent's static ability, as the
/// `apply` of a `Layer::PtModify` static (a negative number is a minus). Cori-Steel Cutter, Embercleave, Grafted Wargear,
/// Doc Ock's Tentacles and the other plain "+N/+M" statics print it.
pub fn plus_pt<const P: i16, const T: i16>(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + P);
    chars.toughness = chars.toughness.map(|v| v + T);
}

/// "Spells you cast" (CR 601.2i, 707.10a): `affected` is a spell on the stack that was cast, from the zone it was cast from
/// (a copy of a spell was cast from none, CR 707.10; a copy of a card that was cast, a prepare spell's, CR 707.12, 722.3c,
/// is a spell cast), and its controller is `source`'s controller. Thrumming Stone and Ral, Crackling Wit's emblem grant
/// their abilities to such spells.
pub fn spell_you_cast(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId) -> bool {
    let s = view.state();
    let obj = s.obj(affected);
    obj.zone.is(ZoneKind::Stack)
        && obj.cast_from.is_some()
        && s.stack.iter().any(|i| i.kind == StackKind::Spell && i.object == affected && i.controller == s.obj(source).controller)
}

/// "Activated abilities of sources with the chosen name can't be activated unless they're mana abilities." (CR 602.5, 605.1a) as a
/// `CardDef::activation_rules` entry: a source is anything with the name (`library::object_has_name`), in any zone it has the
/// ability in. It binds every player, the permanent's controller too; with no name chosen it stops nothing. Petrified Hamlet (a
/// land card name) and Sorcerous Spyglass (any card name) print it.
pub fn chosen_name_activations_barred(q: &dyn Query, asker: &Asker, source: ObjectId, def: &ActivatedDef) -> bool {
    def.is_mana_ability() || !q.obj(asker.source).marks.chosen_card.is_some_and(|name| crate::library::object_has_name(q, source, name))
}

/// "Has [an ability]" (CR 113.6, 613.1f) for the card's delayed ability number 0: a static ability that gives the objects it
/// affects the ability the card lists first in `delayed` (the card as the source has it now, or the card it copies).
/// Astrologian's Planisphere, Decaying Soil, Deconstruction Hammer, Footfall Crater, Hexing Squelcher, Kaldra Compleat,
/// Lavaspur Boots, Nadu, Paradise Mantle, Petrified Hamlet, Quicksilver Dagger, Sheltered by Ghosts, Thorin Oakenshield and Thrumming Stone
/// grant theirs this way.
pub fn grant_delayed_ability(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

/// "Creatures can't attack you unless their controller pays {N} for each creature they control that's attacking you" (CR
/// 508.1d, 508.1h) as a `CardDef::attack_costs` entry: each creature attacking the source's controller costs {N} to
/// declare; one attacking a planeswalker they control or a battle they protect costs nothing (the rulings: "a creature that
/// can't attack you can still attack a planeswalker you control"). Ghostly Prison, Propaganda and Windborn Muse print it
/// with {2}; Dáin, Lord of the Iron Hills with {1}, while its controller has an enduring story.
pub fn attacking_you_costs<const N: u8>(_: &dyn Query, asker: &Asker, _: ObjectId, target: Target) -> Option<ManaCost> {
    (target == Target::Player(asker.controller)).then_some(ManaCost { generic: N, ..ManaCost::ZERO })
}

/// "Creatures can't attack you or planeswalkers you control unless their controller pays {N} for each of those creatures"
/// (CR 508.1d, 508.1h) as a `CardDef::attack_costs` entry: each creature attacking the source's controller or a planeswalker
/// they control costs {N} to declare; one attacking a battle costs nothing. Baird, Steward of Argive and Archon of
/// Absolution print it with {1}.
pub fn attacking_you_or_your_planeswalkers_costs<const N: u8>(q: &dyn Query, asker: &Asker, _: ObjectId, target: Target) -> Option<ManaCost> {
    let yours = match target {
        Target::Player(p) => p == asker.controller,
        Target::Object(id) => q.chars(id).is(CardType::Planeswalker) && q.controller(id) == asker.controller,
        _ => false,
    };
    yours.then_some(ManaCost { generic: N, ..ManaCost::ZERO })
}

/// "Artifacts and creatures entering don't cause abilities to trigger" (CR 603.2, 603.6a) as a `CardDef::trigger_rules` entry:
/// an event that puts an artifact or a creature onto the battlefield, judged as it is there (the continuous effects applied),
/// triggers no ability, the permanent's own enters abilities included. A land that is an artifact or a creature too stops a
/// landfall ability; one that is neither does not (the rulings' Tireless Tracker). Doorkeeper Thrull and Karn, Argent
/// Defender print it.
pub fn artifacts_and_creatures_entering_dont_trigger(q: &dyn Query, _: &Asker, event: &Event, _: &Triggering) -> bool {
    entering(event).is_some_and(|permanent| {
        let chars = q.chars(permanent);
        chars.is(CardType::Artifact) || chars.is(CardType::Creature)
    })
}
