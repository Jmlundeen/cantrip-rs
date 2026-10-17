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

fn tapped_creature_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).tapped)
}

fn creature_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

fn creature_you_dont_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

fn player_or_planeswalker_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Player(_) => true,
        Target::Object(id) => on_battlefield(q, id) && q.chars(id).is(CardType::Planeswalker),
        Target::None | Target::Ability(_) => false,
    }
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
/// "Another target creature", "other target creature" (CR 115.1, 109.5): a creature on the battlefield other than the source
/// of the spell or ability. Arwen, Mortal Queen, Restless Vinestalk, Wolverine, Fierce Fighter, Solitude, Abigale, Eloquent First-Year, Lyna, Veil of Vengeance
/// and Super-Adaptoid print it; a slot of up to one
/// is `up_to_one(other_creature_filter)`.
pub fn other_creature_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && t != Target::Object(asker.source)
}
pub const OTHER_CREATURE: TargetSpec = target(other_creature_filter);
/// "Target tapped creature" (CR 115.1, 608.2b): tapped when chosen and, legality being checked again on resolution, still
/// tapped then. Ajani, Outland Chaperone, Cryogen Relic and The Wandering Emperor print it.
pub const TAPPED_CREATURE: TargetSpec = target(tapped_creature_filter);
fn attacking_creature_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    creature_filter(q, asker, t) && matches!(t, Target::Object(id) if q.is_attacking(id))
}

/// "Target attacking creature" (CR 115.1, 506.4): a creature on the battlefield that is attacking when it is chosen and
/// still is as the ability resolves, or the target is illegal. Raffine, Scheming Seer and Raubahn, Bull of Ala Mhigo print it.
pub const ATTACKING_CREATURE: TargetSpec = target(attacking_creature_filter);
pub const CREATURE_YOU_CONTROL: TargetSpec = target(creature_you_control_filter);
/// "Target creature you don't control" (CR 109.5): any other player's creature, a teammate's included. Winds of Abandon and
/// Bushwhack print it.
pub const CREATURE_YOU_DONT_CONTROL: TargetSpec = target(creature_you_dont_control_filter);
/// "Target player or planeswalker" (CR 115.1, 115.4): any target narrowed to players and planeswalkers. Boros Charm, Kyren
/// Negotiations and Quicksilver Dagger print it.
pub const PLAYER_OR_PLANESWALKER: TargetSpec = target(player_or_planeswalker_filter);
/// "Target creature or planeswalker" (CR 115.1). Unholy Heat and Baleful Mastery print it.
pub const CREATURE_OR_PLANESWALKER: TargetSpec = target(creature_or_planeswalker_filter);
pub const PLAYER: TargetSpec = target(player_filter);
pub const OPPONENT: TargetSpec = target(opponent_filter);
pub const SPELL: TargetSpec = target(spell_filter);
fn spell_or_ability_with_a_single_target_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let stack = &q.state().stack;
    match t {
        Target::Object(id) => stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id && i.target_count == 1),
        Target::Ability(serial) => stack.iter().any(|i| i.kind != StackKind::Spell && i.serial == serial && i.target_count == 1),
        _ => false,
    }
}

/// "Target spell or ability with a single target" (CR 115.1, 115.7a): exactly one target across every slot, counted with
/// repeats, as the item was announced; re-asked as the spell resolves (CR 608.2b). Untimely Malfunction and Bolt Bend print it.
pub const SPELL_OR_ABILITY_WITH_A_SINGLE_TARGET: TargetSpec = target(spell_or_ability_with_a_single_target_filter);
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

/// The player a resolving item targeted in `slot`, if the target is still legal (CR 608.2b) and is a player: `object`'s twin
/// for "target player". Used by `effects::target_player_draws`.
pub fn player(g: &dyn Game, item: &StackItem, slot: usize) -> Option<PlayerId> {
    if slot < item.target_count as usize && g.target_legal(item, slot) { item.targets[slot].player() } else { None }
}

/// Every object target of the item that is still legal (CR 608.2b), in slot order. Fiery Confluence-style spells and
/// Echoing Truth-style "each target" effects read it.
pub fn legal_objects(g: &dyn Game, item: &StackItem) -> Vec<ObjectId> {
    (0..item.target_count as usize).filter_map(|slot| object(g, item, slot)).collect()
}

fn equipment_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if permanent(q, id) && q.controller(id) == asker.controller && q.chars(id).has_subtype(Subtype::Equipment))
}
/// "Target Equipment you control" (CR 115.1, 301.5). Super-Soldier Serum, Raubahn and Thorin print it.
pub const EQUIPMENT_YOU_CONTROL: TargetSpec = target(equipment_you_control_filter);

/// A permanent that is still the object it was on the battlefield (CR 400.7): the start of every battlefield filter.
pub fn permanent(q: &dyn Query, id: ObjectId) -> bool {
    on_battlefield(q, id)
}

fn artifact_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && q.chars(id).is(CardType::Artifact))
}

/// "Target creature or enchantment" (CR 115.1): the filter itself, for a card that adds to it ("an opponent controls").
/// Shredder's Technique prints it as it is and Feed the Swarm adds the opponent.
pub fn creature_or_enchantment(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) || c.is(CardType::Enchantment)
    })
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

/// "Other target nonland permanent" as a filter (CR 115.1): a nonland permanent that is not the source of the ability, for a
/// slot of "up to one" or "up to two". Aang's Iceberg, Phelia, Exuberant Shepherd and Marang River Regent print it.
pub fn other_nonland_permanent_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    nonland_permanent_filter(q, asker, t) && t != Target::Object(asker.source)
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

fn nonbasic_land_an_opponent_controls_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    nonbasic_land_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

fn land_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    land_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

fn card_in_a_graveyard_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if { let o = q.obj(id); o.zone.is(ZoneKind::Graveyard) && o.became.is_none() })
}

fn creature_spell_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    spell_filter(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).is(CardType::Creature))
}

/// "Target creature spell" (CR 115.1, 111.1): a spell on the stack with the card type creature, whatever else it is (an
/// artifact creature, an enchantment creature). Essence Scatter prints it; Stern Scolding's "creature spell with power or
/// toughness 2 or less" starts from it.
pub const CREATURE_SPELL: TargetSpec = target(creature_spell_filter);

fn noncreature_spell_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    spell_filter(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).is(CardType::Creature))
}

fn artifact_or_creature_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    })
}

/// "Target artifact" (CR 115.1). Abrade and Embereth Shieldbreaker print it.
pub const ARTIFACT: TargetSpec = target(artifact_filter);

fn enchantment_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && q.chars(id).is(CardType::Enchantment))
}

/// "Target enchantment" (CR 115.1). Requisition Raid and Heritage Reclamation print it.
pub const ENCHANTMENT: TargetSpec = target(enchantment_filter);
/// "Target artifact or creature you control" (CR 115.1, 205.2a). Concerted Care, Oko, Thief of Crowns (the permanent it exchanges),
/// Ozolith, the Shattered Spire and Three Steps Ahead print it.
pub const ARTIFACT_OR_CREATURE_YOU_CONTROL: TargetSpec = target(artifact_or_creature_you_control_filter);
/// "Target artifact or creature": a permanent that is either (CR 115.1). Oko, Thief of Crowns, Tezzeret, Cruel Captain, Touch
/// the Spirit Realm, Iceberg Titan and Lodestone Needle print it.
pub const ARTIFACT_OR_CREATURE: TargetSpec = target(artifact_or_creature_filter);
/// The filter of `ARTIFACT_OR_CREATURE`, for a slot that is "up to one" of them.
pub fn artifact_or_creature_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if on_battlefield(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    })
}
/// "Target artifact or enchantment". Disenchant and Cathar Commando print it.
pub const ARTIFACT_OR_ENCHANTMENT: TargetSpec = target(artifact_or_enchantment_filter);
/// "Target permanent". Alpha Deathclaw and Argentum Armor print it.
pub const PERMANENT: TargetSpec = target(permanent_filter);
/// "Target nonland permanent". Banishing Knack and Boomerang Basics print it.
pub const NONLAND_PERMANENT: TargetSpec = target(nonland_permanent_filter);

/// "Target nonland permanent with mana value `N` or less" (CR 115.1, 202.3: X in a mana cost is 0 on the battlefield), whoever
/// controls it. Abrupt Decay (3), Vraska, Golgari Queen (3) and Witherbloom Charm (2) print it.
pub fn nonland_permanent_with_mana_value_at_most<const N: u32>(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    nonland_permanent_filter(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= N)
}
/// "Target artifact or enchantment an opponent controls". Cathartic Parting, Witch Enchanter, Wistfulness and Scrapshooter print it.
pub const ARTIFACT_OR_ENCHANTMENT_AN_OPPONENT_CONTROLS: TargetSpec = target(artifact_or_enchantment_an_opponent_controls_filter);
fn permanent_an_opponent_controls_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    permanent_filter(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

/// "Target permanent an opponent controls" (CR 115.1). Assassin's Trophy and the emblem of Teferi, Hero of Dominaria print it.
pub const PERMANENT_AN_OPPONENT_CONTROLS: TargetSpec = target(permanent_an_opponent_controls_filter);
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
/// "Target nonbasic land an opponent controls" (CR 115.1, 205.4a). Demolition Field and Magmatic Hellkite print it.
pub const NONBASIC_LAND_AN_OPPONENT_CONTROLS: TargetSpec = target(nonbasic_land_an_opponent_controls_filter);
/// "Target land you control". Earthbend (Badgermole Cub) and Sea Drake print it.
pub const LAND_YOU_CONTROL: TargetSpec = target(land_you_control_filter);
/// "Target card from your graveyard" (CR 115.1): any card in the graveyard of the ability's controller, still the card that
/// was there (CR 400.7). Eternal Witness, Emeritus of Abundance, Bala Ged Recovery, Cathartic Parting, Tamiyo, Collector of Tales
/// and Quintorius Kand ("any number of target cards from your graveyard") print it.
pub fn any_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &[])
}

/// "Target card from your graveyard": see `any_card_in_your_graveyard`.
pub const CARD_IN_YOUR_GRAVEYARD: TargetSpec = target(any_card_in_your_graveyard);

fn creature_card_in_a_graveyard_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Creature])
}

/// "Target creature card from a graveyard" (CR 115.1): a creature card in any player's graveyard. Reanimate, Necromancy, From
/// the Catacombs, Portal to Phyrexia and Virtue of Persistence print it.
pub const CREATURE_CARD_IN_A_GRAVEYARD: TargetSpec = target(creature_card_in_a_graveyard_filter);

/// "Target card from a graveyard". Emperor of Bones, Ghost Vacuum, Keen-Eyed Curator and Lion Sash print it.
pub const CARD_IN_A_GRAVEYARD: TargetSpec = target(card_in_a_graveyard_filter);
/// "Target noncreature spell". Spell Pierce and Stubborn Denial print it.
pub const NONCREATURE_SPELL: TargetSpec = target(noncreature_spell_filter);

/// "Target [color A] or [color B] spell" (CR 115.1, 105.2): a spell on the stack that is either color, or both. The color
/// words are those of the targeting spell's own text (CR 612), so a text-changing effect on it changes them; `A` and `B`
/// are `Color`s as `u8`. Flashfreeze (red or green) and Precise Redaction (white or black) print it.
fn spell_of_either_color_filter<const A: u8, const B: u8>(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let words = q.words(asker.source);
    spell_filter(q, asker, t)
        && matches!(t, Target::Object(id) if {
            let colors = q.chars(id).colors;
            colors.has(words.color(Color::ALL[A as usize])) || colors.has(words.color(Color::ALL[B as usize]))
        })
}

/// "Target [color A] or [color B] spell": see `spell_of_either_color_filter`. Used as
/// `targets::spell_of_either_color::<{ Color::Red as u8 }, { Color::Green as u8 }>()`.
pub const fn spell_of_either_color<const A: u8, const B: u8>() -> TargetSpec {
    target(spell_of_either_color_filter::<A, B>)
}

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
/// CR 202.3e). Unearth, Carnage, Crimson Chaos, Sevinne's Reclamation and Helping Hand print it.
pub fn card_in_your_graveyard_with_mana_value_at_most(q: &dyn Query, asker: &Asker, t: Target, types: &[CardType], max: u32) -> bool {
    card_in_your_graveyard(q, asker, t, types) && matches!(t, Target::Object(id) if q.mana_value(id) <= max)
}

fn creature_card_with_mana_value_3_or_less_in_your_graveyard_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 3)
}

/// "Target creature card with mana value 3 or less from your graveyard". Unearth, Helping Hand, Carnage, Crimson Chaos,
/// Call a Surprise Witness and Leonardo's Technique print it.
pub const CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD: TargetSpec = target(creature_card_with_mana_value_3_or_less_in_your_graveyard_filter);

/// The card types of a permanent card (CR 110.4a): those that can exist on the battlefield.
pub const PERMANENT_CARD_TYPES: [CardType; 6] =
    [CardType::Artifact, CardType::Battle, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker];

/// Whether these are the characteristics of a permanent card (CR 110.4a). Malevolent Rumble, Moonshadow, Lion Sash,
/// Faith's Reward and The Aesir Escape Valhalla ask it.
pub fn is_permanent_card(c: &Chars) -> bool {
    PERMANENT_CARD_TYPES.iter().any(|&t| c.is(t))
}

/// Whether these are the characteristics of a nonland permanent card (CR 110.4a): an artifact land is a land. Six,
/// Guardian Scalelord and Loamcrafter Faun ask it.
pub fn is_nonland_permanent_card(c: &Chars) -> bool {
    !c.is(CardType::Land) && is_permanent_card(c)
}

/// "Target permanent card in your graveyard" (CR 115.1, 110.4a). Shifting Woodland prints it; Night Nurse asks it of a
/// card put there this turn.
pub fn permanent_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &PERMANENT_CARD_TYPES)
}

/// "Target permanent card in your graveyard". Shifting Woodland prints it.
pub const PERMANENT_CARD_IN_YOUR_GRAVEYARD: TargetSpec = target(permanent_card_in_your_graveyard);

/// "Target nonland permanent card from your graveyard" (CR 115.1, 110.4a). Loamcrafter Faun prints it; Guardian
/// Scalelord asks it of a card with mana value up to its power.
pub fn nonland_permanent_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if card_in_your_graveyard(q, asker, t, &[]) && is_nonland_permanent_card(&q.chars(id)))
}

/// "Target nonland permanent card from your graveyard". Loamcrafter Faun prints it.
pub const NONLAND_PERMANENT_CARD_IN_YOUR_GRAVEYARD: TargetSpec = target(nonland_permanent_card_in_your_graveyard);

/// "Target land card from your graveyard" (CR 115.1, 400.7). Titania, Protector of Argoth, Wrenn and Six (up to one) and
/// Carnivorous Cultivator print it.
pub fn land_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &[CardType::Land])
}

/// "Target land card from your graveyard": see `land_card_in_your_graveyard`.
pub const LAND_CARD_IN_YOUR_GRAVEYARD: TargetSpec = target(land_card_in_your_graveyard);

fn creature_card_in_your_graveyard_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &[CardType::Creature])
}

/// "Target creature card in your graveyard". Recurring Nightmare and Metamorphosis Fanatic print it.
pub const CREATURE_CARD_IN_YOUR_GRAVEYARD: TargetSpec = target(creature_card_in_your_graveyard_filter);

/// "A nontoken artifact" (CR 111.1): an artifact permanent that is a card, not a token. Thopter Foundry sacrifices one as a
/// cost (`filters::nontoken_artifact`) and Lorehold Charm makes each opponent sacrifice one.
pub fn is_nontoken_artifact(q: &dyn Query, id: ObjectId) -> bool {
    !q.obj(id).token && q.chars(id).is(CardType::Artifact)
}

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
    /// A nontoken artifact. Thopter Foundry sacrifices one.
    pub fn nontoken_artifact(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        super::is_nontoken_artifact(q, id)
    }
    /// A Treasure (CR 111.10a): Magda, Brazen Outlaw's and Magda, the Hoardmaster's "Sacrifice five/three Treasures".
    pub fn treasure(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        q.chars(id).has_subtype(Subtype::Treasure)
    }
    /// An artifact or a creature. Deadly Dispute, Eviscerator's Insight and Fanatical Offering sacrifice one.
    pub fn artifact_or_creature(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    }
}
