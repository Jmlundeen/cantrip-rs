use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value." One ability,
/// one effect in two layers (CR 613.6): the Creature type is added in layer 4 ("artifact creature" keeps every prior type and
/// subtype, CR 205.1b; an artifact land stays a land, the ruling), and the power and toughness are set in layer 7b (CR
/// 613.4b) for the permanents the layer-4 part made creatures, even once it is a creature and no longer fits the words
/// (CR 613.6's own example). A later effect that sets its power and toughness wins by timestamp (the ruling: Chimeric
/// Staff). An artifact with mana value 0, an artifact land, is a 0/0 the state-based actions put into the graveyard (CR
/// 704.5f, the ruling). March itself is not an artifact unless something makes it one (Mycosynth Lattice), and then it
/// becomes one of them: the text says "each", not "each other". Which artifacts are noncreature is judged after every
/// other effect of layer 4 that changes that: one that makes a permanent an artifact applies first (CR 613.8a, the
/// dependency: it changes what this applies to).
const TEXT: &str = "Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value.";

/// "Each noncreature artifact", as the layers before have it.
fn noncreature_artifact(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Artifact) && !so_far.is(CardType::Creature)
}

/// "... is an artifact creature" (CR 205.1b, 613.1d, layer 4).
fn creature_too(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Creature);
}

/// "... with power and toughness each equal to its mana value" (CR 613.4b, layer 7b; CR 202.3: a token or a card with no
/// mana cost has mana value 0).
fn mana_value_size(q: &dyn LayerQuery, _: ObjectId, affected: ObjectId, chars: &mut Chars) {
    let value = q.mana_value(affected, chars) as i16;
    chars.power = Some(value);
    chars.toughness = Some(value);
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Type, noncreature_artifact, creature_too, TEXT),
        static_ability_for_each(Layer::PtSet, noncreature_artifact, mana_value_size, TEXT).continues(),
    ],
    ..oracle::MARCH_OF_THE_MACHINES
};
