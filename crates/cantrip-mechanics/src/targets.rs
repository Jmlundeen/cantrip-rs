//! Target slots many cards print (CR 115).

use cantrip_api::*;

fn on_battlefield(q: &dyn Query, id: ObjectId) -> bool {
    q.is_on_battlefield(id) && q.obj(id).became.is_none()
}

fn any_target_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Player(_) => true,
        Target::Object(id) => {
            on_battlefield(q, id) && {
                let c = q.chars(id);
                c.is(CardType::Creature) || c.is(CardType::Planeswalker) || c.is(CardType::Battle)
            }
        }
        Target::None | Target::Ability(_) => false,
    }
}

fn creature_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && q.chars(id).is(CardType::Creature))
}

fn creature_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

fn creature_or_planeswalker_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) || c.is(CardType::Planeswalker)
    })
}

fn player_filter(_: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Player(_))
}

fn opponent_filter(_: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Player(p) if p != asker.controller)
}

fn spell_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id))
}

/// Any target (CR 115.4): a creature, a player, a planeswalker or a battle.
pub const ANY: TargetSpec = target(any_target_filter);
pub const CREATURE: TargetSpec = target(creature_filter);
pub const CREATURE_YOU_CONTROL: TargetSpec = target(creature_you_control_filter);
/// "Target creature or planeswalker" (CR 115.1). Unholy Heat and Baleful Mastery print it.
pub const CREATURE_OR_PLANESWALKER: TargetSpec = target(creature_or_planeswalker_filter);
pub const PLAYER: TargetSpec = target(player_filter);
pub const OPPONENT: TargetSpec = target(opponent_filter);
pub const SPELL: TargetSpec = target(spell_filter);
/// "Target spell or permanent" (CR 115.1): Artificial Evolution and Venser, Shaper Savant print it.
pub const SPELL_OR_PERMANENT: TargetSpec = target(spell_or_permanent_filter);

fn spell_or_permanent_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    spell_filter(q, asker, t) || permanent_filter(q, asker, t)
}

/// Whether a target is a card in a graveyard with one of these types.
pub fn card_in_graveyard(q: &dyn Query, t: Target, types: &[CardType]) -> bool {
    match t {
        Target::Object(id) => {
            let obj = q.obj(id);
            obj.zone.is(ZoneKind::Graveyard) && obj.became.is_none() && types.iter().any(|&ty| q.chars(id).is(ty))
        }
        _ => false,
    }
}

fn instant_or_sorcery_in_your_graveyard_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Instant, CardType::Sorcery])
        && matches!(t, Target::Object(id) if q.obj(id).zone == Zone::graveyard(asker.controller))
}

/// "Target instant or sorcery card in your graveyard" (CR 115.1). Snapcaster Mage and Flashback print it.
pub const INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD: TargetSpec =
    target(instant_or_sorcery_in_your_graveyard_filter);

fn noncreature_artifact_or_enchantment_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Object(id) if q.is_on_battlefield(id) => {
            let c = q.chars(id);
            !c.is(CardType::Creature) && (c.is(CardType::Artifact) || c.is(CardType::Enchantment))
        }
        _ => false,
    }
}

/// "Target noncreature artifact or noncreature enchantment" (CR 115.1). Guerrilla Gorilla and Haywire Mite print it.
pub const NONCREATURE_ARTIFACT_OR_ENCHANTMENT: TargetSpec =
    target(noncreature_artifact_or_enchantment_filter);

/// The object a resolving item targeted in `slot`, if the target is still legal (CR 608.2b) and is an object (not a
/// player). Every effect that acts on "target [thing]" begins here rather than pairing `target_legal` with a pattern
/// match. Used by `destroy_target`, `exile_target` and by most of the cards that act on one target.
pub fn object(g: &dyn Game, item: &StackItem, slot: usize) -> Option<ObjectId> {
    if slot < item.target_count as usize && g.target_legal(item, slot) { item.targets[slot].object() } else { None }
}

/// Every object target of the item that is still legal (CR 608.2b), in slot order. Fiery Confluence-style spells and
/// Echoing Truth-style "each target" effects read it.
pub fn legal_objects(g: &dyn Game, item: &StackItem) -> Vec<ObjectId> {
    (0..item.target_count as usize).filter_map(|slot| object(g, item, slot)).collect()
}

/// A permanent that is still the object it was on the battlefield (CR 400.7): the start of every battlefield filter.
pub fn permanent(q: &dyn Query, id: ObjectId) -> bool {
    on_battlefield(q, id)
}

fn artifact_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && q.chars(id).is(CardType::Artifact))
}

fn artifact_or_enchantment_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Enchantment)
    })
}

fn artifact_or_enchantment_an_opponent_controls_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    artifact_or_enchantment_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

/// "A token you control" as an object filter (a cost's "sacrifice a token", CR 701.21): a token its controller controls,
/// whatever it is. Fountainport sacrifices one.
pub fn token_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.obj(id).token && q.controller(id) == asker.controller
}

fn token_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && token_you_control(q, asker, id))
}

/// "Target token you control" (CR 115.1). Esika's Chariot prints it.
pub const TOKEN_YOU_CONTROL: TargetSpec = target(token_you_control_filter);

fn permanent_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id))
}

fn nonland_permanent_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && !q.chars(id).is(CardType::Land))
}

/// "Target nonland permanent you control" as a filter (CR 115.1), for a card that narrows it further or asks for it
/// as "up to one". Displacer Kitten and Nurturing Pixie print it.
pub fn nonland_permanent_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    nonland_permanent_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

fn nonland_permanent_an_opponent_controls_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    nonland_permanent_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

fn nonland_permanent_an_opponent_controls_mv2_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    nonland_permanent_an_opponent_controls_filter(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 2)
}

fn creature_an_opponent_controls_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

fn land_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && q.chars(id).is(CardType::Land))
}

/// The filter of `NONBASIC_LAND`, for a card that narrows it further.
pub fn nonbasic_land_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    land_filter(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).supertypes.basic)
}

fn land_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    land_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

fn card_in_a_graveyard_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if { let o = q.obj(id); o.zone.is(ZoneKind::Graveyard) && o.became.is_none() })
}

fn noncreature_spell_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    spell_filter(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).is(CardType::Creature))
}

/// "Target artifact" (CR 115.1). Abrade and Embereth Shieldbreaker print it.
pub const ARTIFACT: TargetSpec = target(artifact_filter);
/// "Target artifact or enchantment". Disenchant and Cathar Commando print it.
pub const ARTIFACT_OR_ENCHANTMENT: TargetSpec = target(artifact_or_enchantment_filter);
/// "Target permanent". Alpha Deathclaw and Argentum Armor print it.
pub const PERMANENT: TargetSpec = target(permanent_filter);
/// "Target nonland permanent". Banishing Knack and Boomerang Basics print it.
pub const NONLAND_PERMANENT: TargetSpec = target(nonland_permanent_filter);
/// "Target artifact or enchantment an opponent controls". Cathartic Parting, Witch Enchanter, Wistfulness and Scrapshooter print it.
pub const ARTIFACT_OR_ENCHANTMENT_AN_OPPONENT_CONTROLS: TargetSpec = target(artifact_or_enchantment_an_opponent_controls_filter);
/// "Target nonland permanent an opponent controls". Conclave Tribunal and Sheltered by Ghosts print it.
pub const NONLAND_PERMANENT_AN_OPPONENT_CONTROLS: TargetSpec =
    target(nonland_permanent_an_opponent_controls_filter);
/// "Target nonland permanent an opponent controls with mana value 2 or less" (CR 115.1, 202.3: X on the battlefield is
/// 0). Portable Hole and Seam Rip print it.
pub const NONLAND_PERMANENT_AN_OPPONENT_CONTROLS_WITH_MANA_VALUE_2_OR_LESS: TargetSpec =
    target(nonland_permanent_an_opponent_controls_mv2_filter);
/// "Target creature an opponent controls". Palace Jailer and Mawloc print it.
pub const CREATURE_AN_OPPONENT_CONTROLS: TargetSpec = target(creature_an_opponent_controls_filter);
/// "Target land". Candelabra of Tawnos and Strip Mine print it.
pub const LAND: TargetSpec = target(land_filter);
/// "Target nonbasic land" (CR 115.1, 205.4a). Wasteland prints it.
pub const NONBASIC_LAND: TargetSpec = target(nonbasic_land_filter);
/// "Target land you control". Earthbend (Badgermole Cub) and Sea Drake print it.
pub const LAND_YOU_CONTROL: TargetSpec = target(land_you_control_filter);
/// "Target card from a graveyard". Emperor of Bones, Ghost Vacuum, Keen-Eyed Curator and Lion Sash print it.
pub const CARD_IN_A_GRAVEYARD: TargetSpec = target(card_in_a_graveyard_filter);
/// "Target noncreature spell". Spell Pierce and Stubborn Denial print it.
pub const NONCREATURE_SPELL: TargetSpec = target(noncreature_spell_filter);

/// Whether a target is a card of one of these types in the asker's own graveyard ("target creature card in your
/// graveyard", CR 115.1). Eternal Witness takes any card (`types` empty); Recurring Nightmare and Metamorphosis Fanatic
/// take a creature card.
pub fn card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target, types: &[CardType]) -> bool {
    match t {
        Target::Object(id) => {
            let obj = q.obj(id);
            obj.zone == Zone::graveyard(asker.controller)
                && obj.became.is_none()
                && (types.is_empty() || types.iter().any(|&ty| q.chars(id).is(ty)))
        }
        _ => false,
    }
}

/// `card_in_your_graveyard` with "mana value N or less" (CR 202.3, `Query::mana_value`; X is 0 in a graveyard,
/// CR 107.3b). Unearth, Carnage, Crimson Chaos, Sevinne's Reclamation and Helping Hand print it.
pub fn card_in_your_graveyard_with_mana_value_at_most(q: &dyn Query, asker: &Asker, t: Target, types: &[CardType], max: u32) -> bool {
    card_in_your_graveyard(q, asker, t, types) && matches!(t, Target::Object(id) if q.mana_value(id) <= max)
}

fn creature_card_with_mana_value_3_or_less_in_your_graveyard_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 3)
}

/// "Target creature card with mana value 3 or less from your graveyard". Unearth, Helping Hand, Carnage, Crimson Chaos,
/// Call a Surprise Witness and Leonardo's Technique print it.
pub const CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD: TargetSpec = target(creature_card_with_mana_value_3_or_less_in_your_graveyard_filter);

fn creature_card_in_your_graveyard_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &[CardType::Creature])
}

/// "Target creature card in your graveyard". Recurring Nightmare and Metamorphosis Fanatic print it.
pub const CREATURE_CARD_IN_YOUR_GRAVEYARD: TargetSpec = target(creature_card_in_your_graveyard_filter);

fn small_creature_card_in_your_graveyard_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &[CardType::Creature]) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// "Target creature card with mana value 3 or less from your graveyard" (CR 115.1, 202.3). Unearth and Call a Surprise
/// Witness print it.
pub const CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD: TargetSpec = target(small_creature_card_in_your_graveyard_filter);

/// A card chosen to be sacrificed, discarded or tapped as a cost: the filter shapes `Cost::sacrifice` and
/// `Cost::discard` take. Each is named by what it accepts.
pub mod filters {
    use cantrip_api::*;

    /// A creature. Bankrupt in Blood and Culling the Weak sacrifice one.
    pub fn creature(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        q.chars(id).is(CardType::Creature)
    }
    /// Any card or permanent. Bitter Triumph and Collective Brutality discard one.
    pub fn any(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
        true
    }
    /// A land. Crop Rotation and Baloth Prime sacrifice one.
    pub fn land(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        q.chars(id).is(CardType::Land)
    }
    /// An artifact. Crabomination (emerge) and Time Sieve sacrifice one.
    pub fn artifact(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        q.chars(id).is(CardType::Artifact)
    }
    /// An artifact or a creature. Deadly Dispute, Eviscerator's Insight and Fanatical Offering sacrifice one.
    pub fn artifact_or_creature(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    }
}
