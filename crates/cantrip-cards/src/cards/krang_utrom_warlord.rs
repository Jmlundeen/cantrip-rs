use cantrip_api::*;
use cantrip_oracle as oracle;

const GRANTED: Keywords = Keywords::of(&[Keyword::Flying, Keyword::Trample, Keyword::Indestructible, Keyword::Haste]);

fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    affected != source
        && so_far.is(CardType::Artifact)
        && so_far.is(CardType::Creature)
        && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = Keywords(chars.keywords.0 | GRANTED.0);
}

/// "Other artifact creatures you control have flying, trample, indestructible, and haste."
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, affects, apply, "Other artifact creatures you control have flying, trample, indestructible, and haste.")],
    ..oracle::KRANG_UTROM_WARLORD
};
