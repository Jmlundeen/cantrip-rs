//! Cantrip's wire vocabulary, mirrored: what `src/Cantrip.Protocol` defines in C#, type for type and
//! field for field, so the host emits exactly what the server already sends its clients and the C#
//! side translates nothing.
//!
//! Serialisation is JSON: PascalCase fields as the C# properties are named, enums as their member
//! names, `ViewHandle` as a bare integer, and every union tagged with `$type` naming the C# class.
//! `Cantrip.Server` (C#, `NativeRulesHost`) deserialises these into the real `Cantrip.Protocol` types.
//!
//! Keep this file in step with the C# by hand; a drift test on the C# side compares the two.

use serde::{Deserialize, Serialize};

// --- Handles ---

/// How one viewer refers to one object. Allocated per viewer by `handles::HandleSpace`, never the
/// engine's object id: an id is stable for an object's life and would let a client follow a card
/// through a shuffle. Zero is "none".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewHandle(pub u32);

impl ViewHandle {
    pub const NONE: ViewHandle = ViewHandle(0);

    pub fn is_none(self) -> bool {
        self.0 == 0
    }
}

// --- Vocabulary (Vocabulary.cs) ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TurnStepView {
    Untap,
    Upkeep,
    Draw,
    PrecombatMain,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndOfCombat,
    PostcombatMain,
    End,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhaseView {
    Beginning,
    PrecombatMain,
    Combat,
    PostcombatMain,
    Ending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneKindView {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Stack,
    Exile,
    Command,
    Sideboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CardTypeView {
    Artifact,
    Battle,
    Creature,
    Enchantment,
    Instant,
    Kindred,
    Land,
    Planeswalker,
    Sorcery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SupertypeView {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ColorView {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ManaTypeView {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

impl ManaTypeView {
    /// Index into `PlayerSummaryView::mana`, in the order the C# enum numbers them.
    pub fn index(self) -> usize {
        self as usize
    }

    pub const ALL: [ManaTypeView; 6] = [
        ManaTypeView::White,
        ManaTypeView::Blue,
        ManaTypeView::Black,
        ManaTypeView::Red,
        ManaTypeView::Green,
        ManaTypeView::Colorless,
    ];

    pub fn symbol(self) -> &'static str {
        match self {
            ManaTypeView::White => "{W}",
            ManaTypeView::Blue => "{U}",
            ManaTypeView::Black => "{B}",
            ManaTypeView::Red => "{R}",
            ManaTypeView::Green => "{G}",
            ManaTypeView::Colorless => "{C}",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerStatusView {
    #[default]
    Active,
    Lost,
    Conceded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayNightView {
    #[default]
    Neither,
    Day,
    Night,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameOutcomeView {
    #[default]
    InProgress,
    TeamWon,
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LossReasonView {
    Life,
    EmptyLibrary,
    Poison,
    Conceded,
    Effect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectKindView {
    #[default]
    Card,
    Token,
    SpellCopy,
    AbilityOnStack,
    Emblem,
    CardCopy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionKindView {
    PlayLand,
    CastSpell,
    ActivateAbility,
    UndoManaAbility,
    CancelCasting,
    PayCost,
    DeclineCost,
    AssignMana,
    IncreaseX,
    DecreaseX,
    ConfirmCasting,
    TapToPay,
    TurnFaceUp,
    ExileToPay,
    PayLifeForSymbol,
    TakeCompanion,
    Plot,
    Suspend,
    Foretell,
    /// Unlocks a locked door of the named Room by paying that door's mana cost (CR 709.5e). A special action, at sorcery
    /// timing. `ability_index` says which door: the option's `description` names it and its cost.
    UnlockDoor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardChoicePurposeView {
    #[default]
    Discard,
    PutOnTop,
    KeepLegend,
    Sacrifice,
    Exile,
    ReturnToHand,
    Untap,
    PutOntoBattlefield,
    ExileToPay,
    Keep,
    PutIntoHand,
    Cast,
    TapToPay,
    OrderSimultaneous,
    Destroy,
    Vote,
    PutIntoGraveyard,
    Play,
    Splice,
    PutOnBottom,
    Attach,
    Encode,
    /// What a cost beholds (CR 701.4a): a permanent chosen or a card revealed, nothing lost.
    Behold,
    /// The creature a cost puts -1/-1 counters on (CR 701.68a).
    Blight,
    CastFree,
    CastCopyFree,
    /// Up to one untapped creature tapped as a cost, its power coming off the generic mana of the total cost (harmonize, CR 702.180).
    TapToReduce,
    /// The permanent that gets counters, chosen without targeting: `Counter` and `CounterCount` say which and how many
    /// (empower Jace's loyalty counters, amass's +1/+1 counters).
    PutCounters,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchDestinationView {
    #[default]
    Hand,
    TopOfLibrary,
    Graveyard,
    Battlefield,
    Exile,
    ExileFaceDown,
    Cast,
    BattlefieldTapped,
    CastFree,
    Found,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardDestinationKindView {
    #[default]
    LibraryTop,
    LibraryBottom,
    Graveyard,
    Hand,
    Exile,
    BattlefieldFaceDown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManaSpendFilterView {
    #[default]
    None,
    CreatureSpells,
    InstantOrSorcerySpells,
    ActivatedAbilities,
    ArtifactSpells,
    LegendarySpells,
    NotNonartifactSpells,
    PlaneswalkerSpells,
    /// Abilities of land sources.
    LandAbilities,
    /// Abilities of artifact sources.
    ArtifactAbilities,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManaSpendEffectView {
    #[default]
    None,
    GainsHaste,
    CantBeCountered,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackTargetKindView {
    #[default]
    Player,
    Planeswalker,
    Battle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneChangeReasonView {
    Draw,
    Play,
    Cast,
    Resolve,
    Discard,
    Destroyed,
    Sacrificed,
    Exiled,
    Returned,
    Milled,
    StateBasedAction,
    Countered,
    Searched,
    PutOnLibrary,
    Created,
    Surveilled,
    ShuffledIn,
    Manifested,
    PutIntoHand,
}

// --- The view (PlayerView.cs) ---

/// Everything one player is entitled to see, and nothing else. Seats are numbered from one.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerView {
    pub viewer: i32,
    pub turn: i32,
    pub step: Option<TurnStepView>,
    pub active_player: i32,
    /// Who holds priority, or zero while nobody does.
    pub priority_player: i32,
    pub players: Vec<PlayerSummaryView>,
    /// Objects this viewer can observe, in handle order.
    pub objects: Vec<ObjectView>,
    pub zones: Vec<ZoneView>,
    pub combat: CombatView,
    pub outcome: GameOutcomeView,
    pub winning_team: i32,
    pub day_night: DayNightView,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerSummaryView {
    pub seat: i32,
    pub name: String,
    pub team: i32,
    pub life: i32,
    pub status: PlayerStatusView,
    pub poison_counters: i32,
    pub hand_count: i32,
    pub library_count: i32,
    pub lands_played_this_turn: i32,
    /// Unspent unrestricted mana, indexed by `ManaTypeView`.
    pub mana: [i32; 6],
    pub restricted_mana: Vec<RestrictedManaView>,
    pub is_monarch: bool,
    pub has_initiative: bool,
    pub dungeon: String,
    pub dungeon_room: String,
    pub dungeons_completed: i32,
    pub energy: i32,
    pub has_citys_blessing: bool,
    /// Counters on the player by name (CR 122.1), poison and energy apart.
    pub counters: Vec<CounterView>,
    /// Effects waiting on this player that no object carries, one line each: "The next spell you cast this turn can't be
    /// countered" (CR 611.2c). Public, like the permanent effects on the table.
    pub pending_effects: Vec<String>,
    /// An enduring story (CR 702.195b): public, as the city's blessing is.
    pub has_enduring_story: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RestrictedManaView {
    pub r#type: Option<ManaTypeView>,
    pub amount: i32,
    pub filter: ManaSpendFilterView,
    pub effect: ManaSpendEffectView,
}

/// One object the viewer can see. Characteristics are computed, never printed.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectView {
    pub handle: ViewHandle,
    pub zone: Option<ZoneKindView>,
    /// Seat owning the zone, or zero for a shared zone.
    pub zone_owner: i32,
    pub kind: ObjectKindView,
    pub owner: i32,
    pub controller: i32,
    /// False when the viewer can see that something is there but not what it is.
    pub known: bool,
    /// Content id, for art and oracle text lookup. Empty when not known to the viewer.
    pub card_id: String,
    pub name: String,
    /// `{1}{R}`; `{0}` for a cost of zero; empty for no mana cost at all (CR 202.1b).
    pub mana_cost: String,
    pub mana_value: i32,
    /// Rules text as it currently reads (CR 109.3).
    pub oracle_text: String,
    pub supertypes: Vec<SupertypeView>,
    pub card_types: Vec<CardTypeView>,
    pub subtypes: Vec<String>,
    pub colors: Vec<ColorView>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub tapped: bool,
    pub face_down: bool,
    pub damage_marked: i32,
    pub counters: Vec<CounterView>,
    pub summoning_sick: bool,
    /// The other seats that know this card although it is in a zone they cannot look into.
    pub known_by: Vec<i32>,
    /// What this spell or ability on the stack is aimed at, in printed order.
    pub targets: Vec<SpellTargetView>,
    pub exiled_with: ViewHandle,
    pub produces_mana: bool,
    pub face_down_name: String,
    pub monstrous: bool,
    pub attached_to: ViewHandle,
    pub choices: Vec<ChoiceView>,
    pub exerted: bool,
    pub saddled: bool,
    pub goaded: bool,
    pub other_face: Option<Box<ObjectView>>,
    pub harnessed: bool,
    pub attached_to_player: i32,
    pub granted_abilities: Vec<AbilityTextView>,
    pub spent_lines: Vec<i32>,
    pub fingerprint: i64,
    pub is_copy: bool,
    pub copying_card: Option<Box<ObjectView>>,
    pub prepare_spell: Option<Box<ObjectView>>,
    pub chosen_modes: Vec<String>,
    pub class_level: i32,
    pub protector: i32,
    pub halves: Vec<ObjectView>,
    /// For a Room on the battlefield (CR 709.5): whether its left and right doors are unlocked, in that order. Empty for
    /// anything else. A locked door is unlocked by an activation offered on the Room (CR 709.5e).
    pub unlocked_doors: Vec<bool>,
    pub starting_counter: Option<i32>,
    /// It is every creature type (Changeling, CR 702.73a): `subtypes` then holds all of them, for a client that asks "is it a
    /// Goblin?", and `type_line_subtypes` what its type line prints.
    pub every_creature_type: bool,
    pub type_line_subtypes: Vec<String>,
    /// Permissions to play this card from where it is (CR 601.3, 305.1): an impulse draw's "this turn", a flashback.
    pub play_permissions: Vec<PlayPermissionView>,
    /// A spell that can't be countered because of what was done as it was cast (CR 101.2, 701.6a).
    pub cant_be_countered: bool,
    /// The abilities it printed and has lost (CR 613.1f), as they read; `granted_abilities` holds those it was given.
    pub lost_abilities: Vec<String>,
    /// For an adventurer or omen card, the Adventure or Omen printed in its inset frame (CR 715.2, 720.2), as printed: the
    /// spell it may be cast as. None on the stack for the spell cast as that face, which is the face itself.
    pub inset_spell: Option<Box<ObjectView>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayPermissionView {
    /// The seat that may play the card.
    pub seat: i32,
    /// What it may do and until when, as it reads: "Playable this turn".
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AbilityTextView {
    pub text: String,
    pub spent: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpellTargetView {
    /// Set when the target is a player; zero otherwise.
    pub player: i32,
    pub object: ViewHandle,
    pub text: String,
    pub still_legal: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChoiceView {
    pub key: String,
    pub value: String,
}

/// A kind of counter and how many of it. The name is the card's, lower case: `+1/+1`, `stun`, `lore`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CounterView {
    pub name: String,
    pub count: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZoneView {
    pub kind: Option<ZoneKindView>,
    pub owner: i32,
    pub count: i32,
    /// Contents in zone order for zones the viewer can look into, empty otherwise.
    pub contents: Vec<ViewHandle>,
    pub contents_visible: bool,
    pub known: Vec<KnownCardView>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KnownCardView {
    /// Cards above it in a library, zero for the top; -1 in a zone with no order.
    pub from_top: i32,
    pub card: ObjectView,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CombatView {
    pub attackers: Vec<AttackerView>,
    pub blocks: Vec<BlockView>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttackerView {
    pub attacker: ViewHandle,
    pub target_kind: AttackTargetKindView,
    pub target_player: i32,
    pub target_permanent: ViewHandle,
    pub was_blocked: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockView {
    pub attacker: ViewHandle,
    /// Blockers in damage assignment order (CR 509.2).
    pub blockers: Vec<ViewHandle>,
}

// --- Prompts (Messages.cs) ---

/// Something a spell may target: a player, or an object by handle.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TargetMessage {
    pub player: i32,
    pub object: ViewHandle,
}

impl TargetMessage {
    pub fn player(seat: i32) -> Self {
        TargetMessage { player: seat, object: ViewHandle::NONE }
    }

    pub fn object(handle: ViewHandle) -> Self {
        TargetMessage { player: 0, object: handle }
    }

    pub fn is_player(&self) -> bool {
        self.player != 0
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HybridSymbolView {
    pub colors: Vec<ColorView>,
    /// The life that pays this symbol instead of mana (CR 107.4f), or zero for plain hybrid.
    pub life_cost: i32,
}

/// One thing a player may do at a priority, pay-costs or unless-pays prompt: a prototype the client
/// fills in, never a flat enumeration of concrete actions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActionOptionMessage {
    pub kind: ActionKindView,
    pub source: ViewHandle,
    /// For an activation, which of the source's abilities. For a cast, 0 for the mana cost and n for
    /// the card's nth alternative cost (CR 118.9).
    pub ability_index: i32,
    pub targets_required: i32,
    pub target_candidates: Vec<TargetMessage>,
    /// For a mana ability: the mana it would add.
    pub produces: Vec<ManaTypeView>,
    pub affordable: bool,
    pub produces_of_any_one_color: i32,
    pub has_x: bool,
    pub max_x: i32,
    /// For an activation: how the ability reads, cost and all. For showing; never parsed by a client.
    pub description: String,
    pub life_cost: i32,
    pub damage_to_controller: i32,
    pub color_choices: Vec<ColorView>,
    pub cost_text: String,
    pub targets_optional: bool,
    pub filter: ManaSpendFilterView,
    pub effect: ManaSpendEffectView,
    pub spell_name: String,
    pub produces_of_any_color_each: i32,
    pub sacrifice_candidates: Vec<ViewHandle>,
    pub tap: Option<ViewHandle>,
    pub pays_up_to: i32,
    pub alternative: i32,
    pub life_gained: i32,
    pub granted_by: String,
    /// For a mana ability offered while a cost is paid: its mana is restricted so that none of it may pay this cost
    /// (CR 106.6).
    pub pays_nothing: bool,
}

impl ActionOptionMessage {
    pub fn new(kind: ActionKindView, source: ViewHandle) -> Self {
        ActionOptionMessage {
            kind,
            source,
            ability_index: 0,
            targets_required: 0,
            target_candidates: Vec::new(),
            produces: Vec::new(),
            affordable: true,
            produces_of_any_one_color: 0,
            has_x: false,
            max_x: -1,
            description: String::new(),
            life_cost: 0,
            damage_to_controller: 0,
            color_choices: Vec::new(),
            cost_text: String::new(),
            targets_optional: false,
            filter: ManaSpendFilterView::None,
            effect: ManaSpendEffectView::None,
            spell_name: String::new(),
            produces_of_any_color_each: 0,
            sacrifice_candidates: Vec::new(),
            tap: None,
            pays_up_to: 0,
            alternative: 0,
            life_gained: 0,
            granted_by: String::new(),
            pays_nothing: false,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CardDestinationMessage {
    pub kind: CardDestinationKindView,
    pub owner: i32,
    pub min: i32,
    /// The most that may be put here, or `i32::MAX` for "and the rest".
    pub max: i32,
    pub eligible: Vec<ViewHandle>,
    pub ordered: bool,
    pub randomized: bool,
    pub groups: Vec<Vec<ViewHandle>>,
    pub group_names: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TargetShareMessage {
    pub target: TargetMessage,
    pub amount: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttackMessage {
    pub attacker: ViewHandle,
    /// The seat attacked, when `target_permanent` is none.
    pub target_player: i32,
    pub target_permanent: ViewHandle,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockMessage {
    pub blocker: ViewHandle,
    pub attacker: ViewHandle,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttackRequirementMessage {
    pub attacker: ViewHandle,
    /// Seats it must attack one of; empty for any.
    pub players: Vec<i32>,
}

/// One creature's combat damage and where it may go (CR 510.1).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DamageDivisionMessage {
    pub source: ViewHandle,
    pub damage: i32,
    pub creatures: Vec<ViewHandle>,
    pub lethal: Vec<i32>,
    pub may_assign_to_defender: bool,
    pub defender: i32,
    pub suggested: Vec<i32>,
    pub suggested_to_defender: i32,
    pub defender_permanent: ViewHandle,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DamageShareMessage {
    pub source: ViewHandle,
    /// The creature this share goes to, or none for the player the source is attacking.
    pub creature: ViewHandle,
    pub amount: i32,
}

/// What the server asks a player, one of 21 kinds, each the C# class of the same name.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum PromptPayload {
    PriorityPromptMessage(PriorityPrompt),
    DiscardPromptMessage(DiscardPrompt),
    DeclareAttackersPromptMessage(DeclareAttackersPrompt),
    DeclareBlockersPromptMessage(DeclareBlockersPrompt),
    AssignCombatDamagePromptMessage(AssignCombatDamagePrompt),
    SearchLibraryPromptMessage(SearchLibraryPrompt),
    PayCostsPromptMessage(PayCostsPrompt),
    ChooseCardsPromptMessage(ChooseCardsPrompt),
    UnlessPaysPromptMessage(UnlessPaysPrompt),
    DistributeCardsPromptMessage(DistributeCardsPrompt),
    ChooseTargetPromptMessage(ChooseTargetPrompt),
    MayPromptMessage(MayPrompt),
    ChooseModesPromptMessage(ChooseModesPrompt),
    ChooseReplacementPromptMessage(ChooseReplacementPrompt),
    OrderTriggersPromptMessage(OrderTriggersPrompt),
    NameCardPromptMessage(NameCardPrompt),
    ChoosePlayerPromptMessage(ChoosePlayerPrompt),
    ChooseColorsPromptMessage(ChooseColorsPrompt),
    ChooseDividedTargetsPromptMessage(ChooseDividedTargetsPrompt),
    ChooseStartingPlayerPromptMessage(ChooseStartingPlayerPrompt),
    MulliganPromptMessage(MulliganPrompt),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PriorityPrompt {
    pub player: i32,
    pub options: Vec<ActionOptionMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DiscardPrompt {
    pub player: i32,
    pub count: i32,
    pub maximum_hand_size: i32,
    pub hand: Vec<ViewHandle>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeclareAttackersPrompt {
    pub player: i32,
    pub eligible: Vec<ViewHandle>,
    pub attackable_players: Vec<i32>,
    pub attackable_planeswalkers: Vec<ViewHandle>,
    pub attackable_battles: Vec<ViewHandle>,
    pub requirements: Vec<AttackRequirementMessage>,
    pub forbidden: Vec<AttackMessage>,
}

impl DeclareAttackersPrompt {
    pub fn is_forbidden(&self, attack: &AttackMessage) -> bool {
        self.forbidden.iter().any(|refused| {
            refused.attacker == attack.attacker
                && refused.target_permanent == attack.target_permanent
                && (!refused.target_permanent.is_none() || refused.target_player == attack.target_player)
        })
    }

    /// Every attack `attacker` may be declared making, seats first.
    pub fn attacks_for(&self, attacker: ViewHandle) -> Vec<AttackMessage> {
        self.attackable_players
            .iter()
            .map(|&seat| AttackMessage { attacker, target_player: seat, target_permanent: ViewHandle::NONE })
            .chain(self.attackable_planeswalkers.iter().chain(self.attackable_battles.iter()).map(|&p| AttackMessage {
                attacker,
                target_player: 0,
                target_permanent: p,
            }))
            .filter(|attack| !self.is_forbidden(attack))
            .collect()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeclareBlockersPrompt {
    pub player: i32,
    pub eligible: Vec<ViewHandle>,
    pub attackers: Vec<ViewHandle>,
    pub allowed: Vec<BlockMessage>,
    /// The fewest creatures that may block each of `attackers`, index for index.
    pub minimum_blockers: Vec<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AssignCombatDamagePrompt {
    pub player: i32,
    pub divisions: Vec<DamageDivisionMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SearchLibraryPrompt {
    pub player: i32,
    pub count: i32,
    pub optional: bool,
    pub library: Vec<ViewHandle>,
    pub eligible: Vec<ViewHandle>,
    pub destination: SearchDestinationView,
    pub owner: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PayCostsPrompt {
    pub player: i32,
    pub spell: ViewHandle,
    pub total: String,
    pub remaining: String,
    pub remaining_generic: i32,
    pub remaining_colored: Vec<ColorView>,
    pub options: Vec<ActionOptionMessage>,
    pub has_x: bool,
    pub x: i32,
    pub committed: Vec<RestrictedManaView>,
    pub remaining_hybrid: Vec<HybridSymbolView>,
    pub kicker: String,
    pub multikicker: bool,
    pub buyback: bool,
    pub announces_x_or_kicks: bool,
    pub most_affordable: i32,
    pub x_paid_in_life: bool,
    pub remaining_colorless: i32,
    pub kicker_paid: bool,
    pub variable_kicker: bool,
    pub spends_no_mana: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseCardsPrompt {
    pub player: i32,
    pub owner: i32,
    pub source: ViewHandle,
    pub purpose: CardChoicePurposeView,
    pub cards: Vec<ViewHandle>,
    pub eligible: Vec<ViewHandle>,
    pub count: i32,
    pub up_to: bool,
    pub kind: String,
    pub minimum_power: i32,
    pub powers: Vec<i32>,
    pub minimum_mana_value: i32,
    pub mana_values: Vec<i32>,
    pub maximum_mana_value: i32,
    /// For `PutCounters`, the kind of counter as Oracle text names it ("loyalty", "+1/+1"); empty otherwise.
    pub counter: String,
    /// For `PutCounters`, how many counters the chosen permanent gets.
    pub counter_count: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnlessPaysPrompt {
    pub player: i32,
    pub source: ViewHandle,
    pub subject: ViewHandle,
    pub total: String,
    pub remaining: String,
    pub remaining_generic: i32,
    pub remaining_colored: Vec<ColorView>,
    pub options: Vec<ActionOptionMessage>,
    pub remaining_hybrid: Vec<HybridSymbolView>,
    pub verb: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DistributeCardsPrompt {
    pub player: i32,
    pub source: ViewHandle,
    /// The cards looked at, top card of the library first.
    pub cards: Vec<ViewHandle>,
    pub destinations: Vec<CardDestinationMessage>,
    pub may_shuffle: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseTargetPrompt {
    pub player: i32,
    pub ability: ViewHandle,
    pub source: ViewHandle,
    pub text: String,
    pub candidates: Vec<TargetMessage>,
    pub optional: bool,
    pub count: i32,
    pub targeted: bool,
    pub minimum: i32,
    pub any_number: bool,
    pub for_player: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MayPrompt {
    pub player: i32,
    pub source: ViewHandle,
    pub verb: String,
    pub amount: i32,
    /// The card's own words for a worded "you may" (`Purpose::Ask`), each empty for one that is not.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub question: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub detail: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub yes: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub no: String,
    /// Neither answer is the default a player passing through prompts takes.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_default: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseModesPrompt {
    pub player: i32,
    pub source: ViewHandle,
    pub modes: Vec<String>,
    pub eligible: Vec<i32>,
    pub count: i32,
    pub may_repeat: bool,
    pub additional_cost: bool,
    pub heading: String,
    pub at_most: i32,
    pub affordable: Vec<i32>,
    /// For an optional additional cost that is a keyword action: "Behold" or "Blight"; empty otherwise.
    pub cost_kind: String,
    pub detail: String,
    pub extra_modes_cost: bool,
    pub texts: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseReplacementPrompt {
    pub player: i32,
    pub subject: ViewHandle,
    pub event: String,
    pub sources: Vec<ViewHandle>,
    pub hints: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrderTriggersPrompt {
    pub player: i32,
    pub sources: Vec<ViewHandle>,
    pub texts: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NameCardPrompt {
    pub player: i32,
    pub source: ViewHandle,
    pub owner: i32,
    pub hand: Vec<ViewHandle>,
    pub names: Vec<String>,
    pub suggestion: String,
    /// What the name must be the name of, in the card's words before "card name" (CR 201.4a): "", "nonland" or "land".
    pub kind: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChoosePlayerPrompt {
    pub player: i32,
    pub source: ViewHandle,
    pub candidates: Vec<i32>,
    pub verb: String,
    pub amount: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseColorsPrompt {
    pub player: i32,
    pub source: ViewHandle,
    pub choices: Vec<ColorView>,
    pub count: i32,
    pub may_repeat: bool,
    pub verb: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseDividedTargetsPrompt {
    pub player: i32,
    pub ability: ViewHandle,
    pub source: ViewHandle,
    pub text: String,
    pub candidates: Vec<TargetMessage>,
    pub total: i32,
    pub max_targets: i32,
    pub optional: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChooseStartingPlayerPrompt {
    pub player: i32,
    pub candidates: Vec<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MulliganPrompt {
    pub player: i32,
    pub mulligans: i32,
    pub keeps: i32,
}

// --- Answers ---

/// What a player answers, one of 17 kinds, each the C# class of the same name.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum AnswerPayload {
    PassAnswer(PassAnswer),
    CardsChosenAnswer(CardsChosenAnswer),
    TakeActionAnswer(TakeActionAnswer),
    AttackersDeclaredAnswer(AttackersDeclaredAnswer),
    BlockersDeclaredAnswer(BlockersDeclaredAnswer),
    CombatDamageAssignedAnswer(CombatDamageAssignedAnswer),
    CardsDistributedAnswer(CardsDistributedAnswer),
    TargetsChosenAnswer(TargetsChosenAnswer),
    MayChosenAnswer(MayChosenAnswer),
    ModesChosenAnswer(ModesChosenAnswer),
    ReplacementChosenAnswer(ReplacementChosenAnswer),
    TriggersOrderedAnswer(TriggersOrderedAnswer),
    CancelAnnouncementAnswer(CancelAnnouncementAnswer),
    CardNamedAnswer(CardNamedAnswer),
    PlayerChosenAnswer(PlayerChosenAnswer),
    ColorsChosenAnswer(ColorsChosenAnswer),
    TargetsDividedAnswer(TargetsDividedAnswer),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PassAnswer {}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CancelAnnouncementAnswer {}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CardsChosenAnswer {
    pub cards: Vec<ViewHandle>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TakeActionAnswer {
    pub kind: ActionKindView,
    pub source: ViewHandle,
    pub ability_index: i32,
    pub targets: Vec<TargetMessage>,
    pub color: Option<ColorView>,
    pub colors: Vec<ColorView>,
    pub sacrifice: Option<ViewHandle>,
    pub tap: Option<ViewHandle>,
    pub mana: Option<ManaTypeView>,
    pub filter: ManaSpendFilterView,
    pub effect: ManaSpendEffectView,
}

impl TakeActionAnswer {
    pub fn new(kind: ActionKindView, source: ViewHandle) -> Self {
        TakeActionAnswer {
            kind,
            source,
            ability_index: 0,
            targets: Vec::new(),
            color: None,
            colors: Vec::new(),
            sacrifice: None,
            tap: None,
            mana: None,
            filter: ManaSpendFilterView::None,
            effect: ManaSpendEffectView::None,
        }
    }

    /// The answer that takes `option` as offered, targets and the rest still to fill in.
    pub fn take(option: &ActionOptionMessage) -> Self {
        let mut answer = Self::new(option.kind, option.source);
        answer.ability_index = option.ability_index;
        answer.tap = option.tap;
        answer.filter = option.filter;
        answer.effect = option.effect;
        answer
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttackersDeclaredAnswer {
    pub attacks: Vec<AttackMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockersDeclaredAnswer {
    pub blocks: Vec<BlockMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CombatDamageAssignedAnswer {
    pub shares: Vec<DamageShareMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CardsDistributedAnswer {
    pub placements: Vec<Vec<ViewHandle>>,
    pub shuffle: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TargetsChosenAnswer {
    pub targets: Vec<TargetMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MayChosenAnswer {
    pub accept: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModesChosenAnswer {
    pub modes: Vec<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplacementChosenAnswer {
    pub index: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TriggersOrderedAnswer {
    pub order: Vec<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CardNamedAnswer {
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerChosenAnswer {
    pub player: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ColorsChosenAnswer {
    pub colors: Vec<ColorView>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TargetsDividedAnswer {
    pub shares: Vec<TargetShareMessage>,
}

impl PromptPayload {
    /// The seat being asked.
    pub fn player(&self) -> i32 {
        match self {
            PromptPayload::PriorityPromptMessage(p) => p.player,
            PromptPayload::DiscardPromptMessage(p) => p.player,
            PromptPayload::DeclareAttackersPromptMessage(p) => p.player,
            PromptPayload::DeclareBlockersPromptMessage(p) => p.player,
            PromptPayload::AssignCombatDamagePromptMessage(p) => p.player,
            PromptPayload::SearchLibraryPromptMessage(p) => p.player,
            PromptPayload::PayCostsPromptMessage(p) => p.player,
            PromptPayload::ChooseCardsPromptMessage(p) => p.player,
            PromptPayload::UnlessPaysPromptMessage(p) => p.player,
            PromptPayload::DistributeCardsPromptMessage(p) => p.player,
            PromptPayload::ChooseTargetPromptMessage(p) => p.player,
            PromptPayload::MayPromptMessage(p) => p.player,
            PromptPayload::ChooseModesPromptMessage(p) => p.player,
            PromptPayload::ChooseReplacementPromptMessage(p) => p.player,
            PromptPayload::OrderTriggersPromptMessage(p) => p.player,
            PromptPayload::NameCardPromptMessage(p) => p.player,
            PromptPayload::ChoosePlayerPromptMessage(p) => p.player,
            PromptPayload::ChooseColorsPromptMessage(p) => p.player,
            PromptPayload::ChooseDividedTargetsPromptMessage(p) => p.player,
            PromptPayload::ChooseStartingPlayerPromptMessage(p) => p.player,
            PromptPayload::MulliganPromptMessage(p) => p.player,
        }
    }

    /// The C# class name, which is also the `$type` tag.
    pub fn kind_name(&self) -> &'static str {
        match self {
            PromptPayload::PriorityPromptMessage(_) => "PriorityPromptMessage",
            PromptPayload::DiscardPromptMessage(_) => "DiscardPromptMessage",
            PromptPayload::DeclareAttackersPromptMessage(_) => "DeclareAttackersPromptMessage",
            PromptPayload::DeclareBlockersPromptMessage(_) => "DeclareBlockersPromptMessage",
            PromptPayload::AssignCombatDamagePromptMessage(_) => "AssignCombatDamagePromptMessage",
            PromptPayload::SearchLibraryPromptMessage(_) => "SearchLibraryPromptMessage",
            PromptPayload::PayCostsPromptMessage(_) => "PayCostsPromptMessage",
            PromptPayload::ChooseCardsPromptMessage(_) => "ChooseCardsPromptMessage",
            PromptPayload::UnlessPaysPromptMessage(_) => "UnlessPaysPromptMessage",
            PromptPayload::DistributeCardsPromptMessage(_) => "DistributeCardsPromptMessage",
            PromptPayload::ChooseTargetPromptMessage(_) => "ChooseTargetPromptMessage",
            PromptPayload::MayPromptMessage(_) => "MayPromptMessage",
            PromptPayload::ChooseModesPromptMessage(_) => "ChooseModesPromptMessage",
            PromptPayload::ChooseReplacementPromptMessage(_) => "ChooseReplacementPromptMessage",
            PromptPayload::OrderTriggersPromptMessage(_) => "OrderTriggersPromptMessage",
            PromptPayload::NameCardPromptMessage(_) => "NameCardPromptMessage",
            PromptPayload::ChoosePlayerPromptMessage(_) => "ChoosePlayerPromptMessage",
            PromptPayload::ChooseColorsPromptMessage(_) => "ChooseColorsPromptMessage",
            PromptPayload::ChooseDividedTargetsPromptMessage(_) => "ChooseDividedTargetsPromptMessage",
            PromptPayload::ChooseStartingPlayerPromptMessage(_) => "ChooseStartingPlayerPromptMessage",
            PromptPayload::MulliganPromptMessage(_) => "MulliganPromptMessage",
        }
    }

    /// The simplest legal answer, composed only out of what the prompt itself carries: the mirror of
    /// each C# prompt's `Default()`. What every automatic answerer falls back to.
    pub fn default_answer(&self) -> AnswerPayload {
        match self {
            PromptPayload::PriorityPromptMessage(_) => AnswerPayload::PassAnswer(PassAnswer {}),
            PromptPayload::DiscardPromptMessage(p) => AnswerPayload::CardsChosenAnswer(CardsChosenAnswer {
                cards: p.hand.iter().take(p.count.max(0) as usize).copied().collect(),
            }),
            PromptPayload::DeclareAttackersPromptMessage(p) => AnswerPayload::AttackersDeclaredAnswer(AttackersDeclaredAnswer {
                attacks: p
                    .requirements
                    .iter()
                    .map(|requirement| {
                        if let Some(&seat) = requirement.players.first() {
                            AttackMessage { attacker: requirement.attacker, target_player: seat, target_permanent: ViewHandle::NONE }
                        } else {
                            p.attacks_for(requirement.attacker).into_iter().next().unwrap_or(AttackMessage {
                                attacker: requirement.attacker,
                                target_player: p.attackable_players.first().copied().unwrap_or(0),
                                target_permanent: ViewHandle::NONE,
                            })
                        }
                    })
                    .collect(),
            }),
            PromptPayload::DeclareBlockersPromptMessage(_) => AnswerPayload::BlockersDeclaredAnswer(BlockersDeclaredAnswer::default()),
            PromptPayload::AssignCombatDamagePromptMessage(p) => {
                let mut shares = Vec::new();
                for division in &p.divisions {
                    for (i, creature) in division.creatures.iter().enumerate() {
                        let amount = division.suggested.get(i).copied().unwrap_or(0);
                        if amount > 0 {
                            shares.push(DamageShareMessage { source: division.source, creature: *creature, amount });
                        }
                    }
                    if division.suggested_to_defender > 0 {
                        shares.push(DamageShareMessage {
                            source: division.source,
                            creature: ViewHandle::NONE,
                            amount: division.suggested_to_defender,
                        });
                    }
                }
                AnswerPayload::CombatDamageAssignedAnswer(CombatDamageAssignedAnswer { shares })
            }
            PromptPayload::SearchLibraryPromptMessage(_) => AnswerPayload::CardsChosenAnswer(CardsChosenAnswer::default()),
            PromptPayload::PayCostsPromptMessage(_) => {
                AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::CancelCasting, ViewHandle::NONE))
            }
            PromptPayload::ChooseCardsPromptMessage(p) => AnswerPayload::CardsChosenAnswer(CardsChosenAnswer {
                cards: p.eligible.iter().take(p.count.max(0) as usize).copied().collect(),
            }),
            PromptPayload::UnlessPaysPromptMessage(_) => {
                AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::DeclineCost, ViewHandle::NONE))
            }
            PromptPayload::DistributeCardsPromptMessage(p) => AnswerPayload::CardsDistributedAnswer(CardsDistributedAnswer {
                placements: greedy_distribution(p),
                shuffle: false,
            }),
            PromptPayload::ChooseTargetPromptMessage(p) => AnswerPayload::TargetsChosenAnswer(TargetsChosenAnswer {
                targets: if p.optional { Vec::new() } else { p.candidates.iter().take(p.count.max(0) as usize).cloned().collect() },
            }),
            PromptPayload::MayPromptMessage(_) => AnswerPayload::MayChosenAnswer(MayChosenAnswer { accept: false }),
            PromptPayload::ChooseModesPromptMessage(p) => {
                let count = p.count.max(0) as usize;
                let modes = if p.may_repeat {
                    (0..count).map(|i| p.eligible[i.min(p.eligible.len().saturating_sub(1))]).collect()
                } else {
                    // Affordable ones first, then the rest, each group in printed order.
                    let mut preferred: Vec<i32> = p.eligible.iter().copied().filter(|m| p.affordable.is_empty() || p.affordable.contains(m)).collect();
                    preferred.extend(p.eligible.iter().copied().filter(|m| !p.affordable.is_empty() && !p.affordable.contains(m)));
                    preferred.into_iter().take(count).collect()
                };
                AnswerPayload::ModesChosenAnswer(ModesChosenAnswer { modes })
            }
            PromptPayload::ChooseReplacementPromptMessage(_) => AnswerPayload::ReplacementChosenAnswer(ReplacementChosenAnswer { index: 0 }),
            PromptPayload::OrderTriggersPromptMessage(p) => AnswerPayload::TriggersOrderedAnswer(TriggersOrderedAnswer {
                order: (0..p.sources.len() as i32).collect(),
            }),
            PromptPayload::NameCardPromptMessage(p) => AnswerPayload::CardNamedAnswer(CardNamedAnswer { name: p.suggestion.clone() }),
            PromptPayload::ChoosePlayerPromptMessage(p) => {
                AnswerPayload::PlayerChosenAnswer(PlayerChosenAnswer { player: p.candidates.first().copied().unwrap_or(0) })
            }
            PromptPayload::ChooseColorsPromptMessage(p) => AnswerPayload::ColorsChosenAnswer(ColorsChosenAnswer {
                colors: if p.may_repeat {
                    vec![p.choices.first().copied().unwrap_or(ColorView::White); p.count.max(0) as usize]
                } else {
                    p.choices.iter().take(p.count.max(0) as usize).copied().collect()
                },
            }),
            PromptPayload::ChooseDividedTargetsPromptMessage(p) => AnswerPayload::TargetsDividedAnswer(TargetsDividedAnswer {
                shares: match p.candidates.first() {
                    Some(first) if p.total > 0 => vec![TargetShareMessage { target: first.clone(), amount: p.total }],
                    _ => Vec::new(),
                },
            }),
            PromptPayload::ChooseStartingPlayerPromptMessage(p) => AnswerPayload::PlayerChosenAnswer(PlayerChosenAnswer { player: p.player }),
            PromptPayload::MulliganPromptMessage(_) => AnswerPayload::MayChosenAnswer(MayChosenAnswer { accept: false }),
        }
    }
}

/// Requirements met in order, then everything left over into the first destination that will take
/// it: the placement the engine would fall back to (`CardDistribution.Greedy` in C#).
fn greedy_distribution(prompt: &DistributeCardsPrompt) -> Vec<Vec<ViewHandle>> {
    let mut placements: Vec<Vec<ViewHandle>> = vec![Vec::new(); prompt.destinations.len()];
    let mut left: Vec<ViewHandle> = prompt.cards.clone();
    for (i, destination) in prompt.destinations.iter().enumerate() {
        while (placements[i].len() as i32) < destination.min {
            let Some(pos) = left.iter().position(|c| destination.eligible.is_empty() || destination.eligible.contains(c)) else { break };
            placements[i].push(left.remove(pos));
        }
    }
    for card in left {
        if let Some(i) = prompt.destinations.iter().enumerate().position(|(i, d)| {
            (placements[i].len() as i32) < d.max && (d.eligible.is_empty() || d.eligible.contains(&card))
        }) {
            placements[i].push(card);
        } else if let Some(first) = placements.first_mut() {
            first.push(card);
        }
    }
    placements
}

// --- Events (Messages.cs, GameEventView) ---

/// Something that happened, as a viewer may be told it: one of 29 kinds, each the C# class of the
/// same name.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum GameEventView {
    ObjectMovedEvent(ObjectMovedEvent),
    LifeChangedEvent(LifeChangedEvent),
    DamageMarkedEvent(DamageMarkedEvent),
    TapChangedEvent(TapChangedEvent),
    StepBeganEvent(StepBeganEvent),
    SpellCastEvent(SpellCastEvent),
    SpellFizzledEvent(SpellFizzledEvent),
    LibraryShuffledEvent(LibraryShuffledEvent),
    CardsRevealedEvent(CardsRevealedEvent),
    EventsRetractedEvent(EventsRetractedEvent),
    LibraryArrangedEvent(LibraryArrangedEvent),
    AbilityActivatedEvent(AbilityActivatedEvent),
    TokenCreatedEvent(TokenCreatedEvent),
    CountersPutEvent(CountersPutEvent),
    ExtraTurnAddedEvent(ExtraTurnAddedEvent),
    PlayerLostEvent(PlayerLostEvent),
    GameEndedEvent(GameEndedEvent),
    TransformedEvent(TransformedEvent),
    DieRolledEvent(DieRolledEvent),
    ExtraPhaseAddedEvent(ExtraPhaseAddedEvent),
    DayNightBecameEvent(DayNightBecameEvent),
    MonarchBecameEvent(MonarchBecameEvent),
    RandomPlayerChosenEvent(RandomPlayerChosenEvent),
    StartingPlayerChosenEvent(StartingPlayerChosenEvent),
    MulliganTakenEvent(MulliganTakenEvent),
    HandKeptEvent(HandKeptEvent),
    AbilityResolvedEvent(AbilityResolvedEvent),
    CoinFlippedEvent(CoinFlippedEvent),
    TurnEndedEvent(TurnEndedEvent),
    CardsLookedAtEvent(CardsLookedAtEvent),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectMovedEvent {
    /// None when the viewer is not entitled to know which card moved.
    pub object: ViewHandle,
    pub from: ZoneKindView,
    pub from_owner: i32,
    pub to: ZoneKindView,
    pub to_owner: i32,
    pub reason: ZoneChangeReasonView,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LifeChangedEvent {
    pub player: i32,
    pub delta: i32,
    pub new_total: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DamageMarkedEvent {
    pub object: ViewHandle,
    pub amount: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TapChangedEvent {
    pub object: ViewHandle,
    pub tapped: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StepBeganEvent {
    pub turn: i32,
    pub step: TurnStepView,
    pub active_player: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpellCastEvent {
    pub spell: ViewHandle,
    pub controller: i32,
    pub targets: Vec<TargetMessage>,
    pub x: Option<i32>,
    pub modes: Vec<String>,
    pub kicked: i32,
    pub buyback: bool,
    pub gift_promised_to: Option<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpellFizzledEvent {
    pub spell: ViewHandle,
    pub controller: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LibraryShuffledEvent {
    pub player: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CardsRevealedEvent {
    pub player: i32,
    pub cards: Vec<ViewHandle>,
}

/// A player looked at hidden cards (a library's top card, a card of a hand): the cards are known to that player alone.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CardsLookedAtEvent {
    pub player: i32,
    pub cards: Vec<ViewHandle>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EventsRetractedEvent {
    pub sequences: Vec<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LibraryArrangedEvent {
    pub player: i32,
    pub on_top: i32,
    pub on_bottom: i32,
    pub shuffled: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AbilityActivatedEvent {
    pub ability: ViewHandle,
    pub source: ViewHandle,
    pub controller: i32,
    pub text: String,
    pub targets: Vec<TargetMessage>,
    pub triggered: bool,
    pub modes: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AbilityResolvedEvent {
    pub ability: ViewHandle,
    pub source: ViewHandle,
    pub controller: i32,
    pub first: i64,
    pub targets: Vec<TargetMessage>,
    pub power_toughness: Vec<PowerToughnessChangeView>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerToughnessChangeView {
    pub object: ViewHandle,
    pub power: i32,
    pub toughness: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TokenCreatedEvent {
    pub object: ViewHandle,
    pub controller: i32,
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CountersPutEvent {
    pub object: ViewHandle,
    pub name: String,
    pub count: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExtraTurnAddedEvent {
    pub player: i32,
    pub count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerLostEvent {
    pub player: i32,
    pub reason: LossReasonView,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameEndedEvent {
    pub outcome: GameOutcomeView,
    pub winning_team: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransformedEvent {
    pub object: ViewHandle,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DieRolledEvent {
    pub player: i32,
    pub sides: i32,
    pub result: i32,
}

/// An effect ended the turn of `player` (CR 724.1): the stack is exiled and the turn goes straight to its cleanup step.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TurnEndedEvent {
    pub player: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExtraPhaseAddedEvent {
    pub phase: PhaseView,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DayNightBecameEvent {
    pub designation: DayNightView,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MonarchBecameEvent {
    pub player: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RandomPlayerChosenEvent {
    pub controller: i32,
    pub selected_player: i32,
    pub source: ViewHandle,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartingPlayerChosenEvent {
    pub chooser: i32,
    pub player: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MulliganTakenEvent {
    pub player: i32,
    pub count: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CoinFlippedEvent {
    pub player: i32,
    pub won: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HandKeptEvent {
    pub player: i32,
    pub cards: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_are_tagged_with_the_csharp_class_name() {
        let prompt = PromptPayload::MayPromptMessage(MayPrompt { player: 1, source: ViewHandle(7), verb: "SearchLibrary".into(), amount: 0, ..MayPrompt::default() });
        let json = serde_json::to_string(&prompt).unwrap();
        assert_eq!(json, r#"{"$type":"MayPromptMessage","Player":1,"Source":7,"Verb":"SearchLibrary","Amount":0}"#);
        let back: PromptPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.player(), 1);
    }

    #[test]
    fn answers_round_trip() {
        let answer = AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::PlayLand, ViewHandle(3)));
        let json = serde_json::to_string(&answer).unwrap();
        assert!(json.starts_with(r#"{"$type":"TakeActionAnswer","Kind":"PlayLand","Source":3"#), "{json}");
        let _: AnswerPayload = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn default_answers_never_name_what_was_not_offered() {
        let prompt = PromptPayload::ChooseTargetPromptMessage(ChooseTargetPrompt {
            player: 1,
            candidates: vec![TargetMessage::player(2)],
            optional: false,
            count: 1,
            ..Default::default()
        });
        match prompt.default_answer() {
            AnswerPayload::TargetsChosenAnswer(t) => assert_eq!(t.targets, vec![TargetMessage::player(2)]),
            other => panic!("{other:?}"),
        }
    }
}
