//! Why an action was rejected ([`RunError`]) and its stable code for
//! frontends ([`RunErrorCode`]).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::deck::{DealtCard, DeckError};
use crate::game::GameError;

/// Why an action was rejected. The run is unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunError {
    /// The action does not fit the current phase.
    WrongPhase,
    /// The game rejected the action.
    Game(GameError<DealtCard>),
    /// No passes left in this series. Never returned since Einmischen
    /// replaced passing (01.11.2026); kept so that the error codes stay
    /// stable.
    NoPassesLeft,
    /// The boss demands a hand game.
    HandRequired,
    /// Null is not available. Never returned since ADR 0012 (every
    /// character may declare Null); kept so that the error codes stay
    /// stable.
    NullNotAvailable,
    /// No Einmischen left in this game.
    NoRedealLeft,
    /// Kontra must be answered first.
    AwaitingKontraAnswer,
    /// There is no Kontra to answer.
    NoKontra,
    /// The declarer does not hold this card instance.
    UnknownCard(u32),
    /// No item in this slot.
    InvalidSlot,
    /// Not enough Groschen.
    NotEnoughCoins,
    /// All slots are taken.
    NoFreeSlot,
    /// The deck rejected the change.
    Deck(DeckError),
    /// This consumable needs a target card.
    TargetRequired,
    /// No card modification can be added to this card.
    NoModAvailable,
    /// The content does not know this id.
    UnknownContent(String),
    /// A [`RunStart`](super::RunStart) names a section or series that does not exist.
    InvalidStart,
    /// The character cannot declare this game (the Nullspieler only Null,
    /// nobody Ramsch).
    GameNotAvailable,
    /// "Der Angeber" demands an Ouvert game.
    OuvertRequired,
    /// No skat swap ("Die Oma") left in this series, or none granted.
    NoSkatSwapLeft,
    /// No matching card lies in the skat.
    NothingToTake,
    /// The Ramsch of the series cannot be dealt again.
    RamschRequired,
    /// A defender is to move: advance the AI ([`Run::advance_ai`](super::Run::advance_ai)) before
    /// the player acts again.
    AiToMove,
    /// "Blue Curaçao" cannot repaint this card into this suit: a jack, a blank,
    /// or the suit it already has.
    CannotChalk,
    /// The rest of the game can still change its score
    /// ([`RunAction::SettleGame`](super::RunAction::SettleGame)).
    NotDecided,
    /// "Reue": there is no Einmischen of this game to take back.
    NoRedealToUndo,
    /// "Lockvogel": no lead can be demanded in this game. Never returned
    /// since the relic was removed (02.11.2026); kept so that the error
    /// codes stay stable.
    NoLeadDemandLeft,
    /// A Spelunke's malus locks the Schnäpse in this pub (ADR 0016).
    ConsumablesLocked,
    /// "Konterbier" until ADR 0019 (never returned since, kept for stable
    /// codes): no life was missing.
    LivesFull,
    /// The Kartenmaler waits for the player's choice
    /// ([`RunAction::ChoosePaint`](super::RunAction::ChoosePaint)); nothing else goes before it.
    PainterPending,
    /// An "Untot" card waits for the player's three cards
    /// ([`RunAction::ChooseRiseCards`](super::RunAction::ChooseRiseCards)); nothing else goes before it.
    RisePending,
    /// The cards for an "Untot" card are not three different cards of the
    /// player's hand, or no "Untot" card waits for them.
    InvalidRiseCards,
    /// The Kartenmaler, "Karte reinigen" or "Neu auslosen" was used at
    /// this Tresen already.
    AlreadyUsed,
    /// "Bruderherz": a curse on his edition cannot be cleansed.
    CurseStays,
    /// The consumable does nothing now: "Klarer" with an account of 0 or
    /// more, "Eierlikör" with an account of half the Zielwert or more.
    ConditionNotMet,
    /// Not in a boss pub ("Absacker").
    NotInBossPub,
    /// The Schnaps of a won pub waits for the player's choice
    /// ([`RunAction::ChooseSchnappsReward`](super::RunAction::ChooseSchnappsReward)); nothing else goes before it.
    SchnappsRewardPending,
    /// The card reward of a won pub waits for the player
    /// ([`RunAction::TakeCardReward`](super::RunAction::TakeCardReward), [`RunAction::SkipCardReward`](super::RunAction::SkipCardReward));
    /// nothing else goes before it.
    CardRewardPending,
}

/// Defines [`RunErrorCode`] and its [`RunErrorCode::as_str`] from one
/// list of variants, so that the text of a code is always its name (equal
/// to its serialized form) and no arm can be forgotten.
macro_rules! error_codes {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($(#[$vmeta:meta])* $variant:ident,)*
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $($(#[$vmeta])* $variant,)*
        }

        impl $name {
            /// The code as text, equal to its serialized form.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant),)*
                }
            }

            /// Every code, in declaration order.
            #[cfg(test)]
            const ALL: &'static [$name] = &[$($name::$variant,)*];
        }
    };
}

error_codes! {
    /// Stable code of a [`RunError`], for frontends: serialized as its name
    /// (`"NotEnoughCoins"`), never renamed once released. A frontend maps the
    /// code to its own text; the English message of [`RunError`]'s `Display`
    /// is for logs only.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
    pub enum RunErrorCode {
        /// The action does not fit the current phase of the run.
        WrongPhase,
        /// No passes left (never since Einmischen replaced passing, kept for
        /// stable codes).
        NoPassesLeft,
        /// The boss demands a hand game.
        HandRequired,
        /// Null is not available (never since ADR 0012, kept for stable codes).
        NullNotAvailable,
        /// No Einmischen left in this game.
        NoRedealLeft,
        /// Kontra must be answered first.
        AwaitingKontraAnswer,
        /// There is no Kontra to answer.
        NoKontra,
        /// The declarer does not hold this card.
        UnknownCard,
        /// No item in this slot.
        InvalidSlot,
        /// Not enough Groschen.
        NotEnoughCoins,
        /// All slots are taken.
        NoFreeSlot,
        /// The consumable needs a target card.
        TargetRequired,
        /// No card modification can be added to this card.
        NoModAvailable,
        /// The content does not know this id.
        UnknownContent,
        /// The start of the run names a section or series that does not exist.
        InvalidStart,
        /// The game rejected the action in its current phase.
        GameWrongPhase,
        /// It is another seat's turn.
        NotYourTurn,
        /// The card is not in the hand.
        CardNotInHand,
        /// The card does not follow suit although the hand could.
        MustFollowSuit,
        /// The two discards are not two distinct cards of the hand.
        InvalidDiscard,
        /// The cards do not form a legal deal.
        InvalidDeal,
        /// A trick is in progress.
        TrickInProgress,
        /// The declaration does not fit the skat: before it is picked up only
        /// hand games, after it only games without hand levels.
        InvalidDeclaration,
        /// The deck has no card with this id.
        UnknownDeckCard,
        /// The deck cannot shrink below 32 cards (no longer produced since
        /// ADR 0015: no card leaves the deck).
        DeckAtMinimum,
        /// "Frisches Deck": the card carries no modification.
        NothingToCleanse,
        /// Blanks carry no modifications.
        BlankCannotBeModified,
        /// The card already carries this modification and it cannot be
        /// raised.
        AlreadyModified,
        /// The card carries a curse and takes no modification (DESIGN 7.2).
        CardCursed,
        /// The modification on the card is at its highest value.
        ModAtMaximum,
        /// The card is not in the deck.
        MissingCard,
        /// The character cannot declare this game.
        GameNotAvailable,
        /// An Ouvert game is required.
        OuvertRequired,
        /// No skat swap left.
        NoSkatSwapLeft,
        /// No matching card in the skat.
        NothingToTake,
        /// The skat cannot be swapped with now.
        InvalidSkatSwap,
        /// The Ramsch cannot be dealt again.
        RamschRequired,
        /// The card must not be discarded ("Ramponiert").
        DiscardForbidden,
        /// A "Doppelgänger" mirrors only as the last card of a trick.
        NotLastCard,
        /// A defender is to move; the player has to wait.
        AiToMove,
        /// "Blue Curaçao" cannot repaint this card into this suit.
        CannotChalk,
        /// The rest of the game can still change its score; it cannot be
        /// settled ([`RunAction::SettleGame`](super::RunAction::SettleGame)).
        NotDecided,
        /// "Reue": no Einmischen of this game to take back.
        NoRedealToUndo,
        /// "Lockvogel" (removed on 02.11.2026, never returned, kept for stable
        /// codes): the lead did not meet the demand.
        MustLeadDemanded,
        /// "Lockvogel" (never returned, kept for stable codes): no lead could
        /// be demanded.
        InvalidLeadDemand,
        /// "Lockvogel" (never returned, kept for stable codes): none left.
        NoLeadDemandLeft,
        /// A Spelunke's malus locks the Schnäpse.
        ConsumablesLocked,
        /// No life was missing (never returned since ADR 0019).
        LivesFull,
        /// The Kartenmaler waits for the player's choice.
        PainterPending,
        /// An "Untot" card waits for the player's three cards.
        RisePending,
        /// Not three different cards of the hand, or nothing waits for them.
        InvalidRiseCards,
        /// The Kartenmaler, "Karte reinigen" or "Neu auslosen" was used at
        /// this Tresen already.
        AlreadyUsed,
        /// The character's curses cannot be cleansed ("Bruderherz").
        CurseStays,
        /// The consumable does nothing now ("Klarer" without a minus,
        /// "Eierlikör" without being behind).
        ConditionNotMet,
        /// Not in a boss pub ("Absacker").
        NotInBossPub,
        /// The Schnaps of a won pub waits for the player's choice
        /// ([`RunAction::ChooseSchnappsReward`](super::RunAction::ChooseSchnappsReward)); nothing else goes before it.
        SchnappsRewardPending,
        /// The card reward of a won pub waits for the player.
        CardRewardPending,
    }
}

impl RunError {
    /// The stable code of this error.
    #[must_use]
    pub const fn code(&self) -> RunErrorCode {
        match self {
            RunError::WrongPhase => RunErrorCode::WrongPhase,
            RunError::Game(e) => match e {
                GameError::WrongPhase { .. } => RunErrorCode::GameWrongPhase,
                GameError::NotYourTurn { .. } => RunErrorCode::NotYourTurn,
                GameError::CardNotInHand { .. } => RunErrorCode::CardNotInHand,
                GameError::MustFollowSuit { .. } => RunErrorCode::MustFollowSuit,
                GameError::InvalidDiscard { .. } => RunErrorCode::InvalidDiscard,
                GameError::InvalidDeal => RunErrorCode::InvalidDeal,
                GameError::TrickInProgress => RunErrorCode::TrickInProgress,
                GameError::InvalidDeclaration { .. } => RunErrorCode::InvalidDeclaration,
                GameError::InvalidSkatSwap => RunErrorCode::InvalidSkatSwap,
                GameError::DiscardForbidden { .. } => RunErrorCode::DiscardForbidden,
                GameError::NotLastCard { .. } => RunErrorCode::NotLastCard,
            },
            RunError::NoPassesLeft => RunErrorCode::NoPassesLeft,
            RunError::HandRequired => RunErrorCode::HandRequired,
            RunError::NullNotAvailable => RunErrorCode::NullNotAvailable,
            RunError::NoRedealLeft => RunErrorCode::NoRedealLeft,
            RunError::AwaitingKontraAnswer => RunErrorCode::AwaitingKontraAnswer,
            RunError::NoKontra => RunErrorCode::NoKontra,
            RunError::UnknownCard(_) => RunErrorCode::UnknownCard,
            RunError::InvalidSlot => RunErrorCode::InvalidSlot,
            RunError::NotEnoughCoins => RunErrorCode::NotEnoughCoins,
            RunError::NoFreeSlot => RunErrorCode::NoFreeSlot,
            RunError::Deck(e) => match e {
                DeckError::UnknownCard(_) => RunErrorCode::UnknownDeckCard,
                DeckError::NothingToCleanse(_) => RunErrorCode::NothingToCleanse,
                DeckError::BlankCannotBeModified => RunErrorCode::BlankCannotBeModified,
                DeckError::AlreadyModified(_) => RunErrorCode::AlreadyModified,
                DeckError::Cursed(_) => RunErrorCode::CardCursed,
                DeckError::AtMaximum(_) => RunErrorCode::ModAtMaximum,
                DeckError::UnknownMod(_) => RunErrorCode::UnknownContent,
                DeckError::MissingCard(_) => RunErrorCode::MissingCard,
            },
            RunError::TargetRequired => RunErrorCode::TargetRequired,
            RunError::NoModAvailable => RunErrorCode::NoModAvailable,
            RunError::UnknownContent(_) => RunErrorCode::UnknownContent,
            RunError::InvalidStart => RunErrorCode::InvalidStart,
            RunError::GameNotAvailable => RunErrorCode::GameNotAvailable,
            RunError::OuvertRequired => RunErrorCode::OuvertRequired,
            RunError::NoSkatSwapLeft => RunErrorCode::NoSkatSwapLeft,
            RunError::NothingToTake => RunErrorCode::NothingToTake,
            RunError::RamschRequired => RunErrorCode::RamschRequired,
            RunError::AiToMove => RunErrorCode::AiToMove,
            RunError::CannotChalk => RunErrorCode::CannotChalk,
            RunError::NotDecided => RunErrorCode::NotDecided,
            RunError::NoRedealToUndo => RunErrorCode::NoRedealToUndo,
            RunError::NoLeadDemandLeft => RunErrorCode::NoLeadDemandLeft,
            RunError::ConsumablesLocked => RunErrorCode::ConsumablesLocked,
            RunError::LivesFull => RunErrorCode::LivesFull,
            RunError::PainterPending => RunErrorCode::PainterPending,
            RunError::RisePending => RunErrorCode::RisePending,
            RunError::InvalidRiseCards => RunErrorCode::InvalidRiseCards,
            RunError::AlreadyUsed => RunErrorCode::AlreadyUsed,
            RunError::CurseStays => RunErrorCode::CurseStays,
            RunError::ConditionNotMet => RunErrorCode::ConditionNotMet,
            RunError::NotInBossPub => RunErrorCode::NotInBossPub,
            RunError::SchnappsRewardPending => RunErrorCode::SchnappsRewardPending,
            RunError::CardRewardPending => RunErrorCode::CardRewardPending,
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::WrongPhase => f.write_str("the action does not fit the current phase"),
            RunError::Game(e) => write!(f, "game: {e}"),
            RunError::NoPassesLeft => f.write_str("no passes left in this series"),
            RunError::HandRequired => f.write_str("the boss demands a hand game"),
            RunError::NullNotAvailable => f.write_str("the declarer does not play Null"),
            RunError::NoRedealLeft => f.write_str("no Einmischen left in this game"),
            RunError::AwaitingKontraAnswer => f.write_str("Kontra must be answered first"),
            RunError::NoKontra => f.write_str("there is no Kontra to answer"),
            RunError::UnknownCard(id) => write!(f, "no card with id {id} in hand"),
            RunError::InvalidSlot => f.write_str("no item in this slot"),
            RunError::NotEnoughCoins => f.write_str("not enough Groschen"),
            RunError::NoFreeSlot => f.write_str("all slots are taken"),
            RunError::Deck(e) => write!(f, "deck: {e}"),
            RunError::TargetRequired => f.write_str("this consumable needs a target card"),
            RunError::NoModAvailable => f.write_str("no modification can be added to this card"),
            RunError::UnknownContent(id) => write!(f, "unknown content {id}"),
            RunError::InvalidStart => f.write_str("the run cannot start at this series"),
            RunError::GameNotAvailable => f.write_str("this character cannot declare that game"),
            RunError::OuvertRequired => f.write_str("an Ouvert game is required"),
            RunError::NoSkatSwapLeft => f.write_str("no skat swap left in this series"),
            RunError::NothingToTake => f.write_str("no matching card lies in the skat"),
            RunError::RamschRequired => f.write_str("the Ramsch must be played"),
            RunError::AiToMove => f.write_str("a defender is to move"),
            RunError::CannotChalk => f.write_str("this card cannot be repainted into this suit"),
            RunError::NotDecided => f.write_str("the rest of the game is not decided"),
            RunError::NoRedealToUndo => f.write_str("no Einmischen to take back"),
            RunError::NoLeadDemandLeft => f.write_str("no lead can be demanded in this game"),
            RunError::ConsumablesLocked => f.write_str("the Schnäpse are locked in this pub"),
            RunError::LivesFull => f.write_str("no life is missing"),
            RunError::PainterPending => f.write_str("the Kartenmaler waits for a choice"),
            RunError::RisePending => f.write_str("an undead card waits for three cards"),
            RunError::InvalidRiseCards => {
                f.write_str("not three different cards of the hand for an undead card")
            }
            RunError::AlreadyUsed => f.write_str("already used at this Tresen"),
            RunError::CurseStays => f.write_str("the character's curses cannot be cleansed"),
            RunError::ConditionNotMet => f.write_str("the consumable would do nothing now"),
            RunError::NotInBossPub => f.write_str("not in a boss pub"),
            RunError::SchnappsRewardPending => {
                f.write_str("the Schnaps of the won pub waits for a choice")
            }
            RunError::CardRewardPending => {
                f.write_str("the card reward of the won pub waits for a choice")
            }
        }
    }
}

impl std::error::Error for RunError {}

impl From<GameError<DealtCard>> for RunError {
    fn from(e: GameError<DealtCard>) -> RunError {
        RunError::Game(e)
    }
}

impl From<DeckError> for RunError {
    fn from(e: DeckError) -> RunError {
        RunError::Deck(e)
    }
}

#[cfg(test)]
mod tests {
    use super::RunErrorCode;

    /// The texts of the codes, as frontends know them. A code is never
    /// renamed once released; a new code goes at the end.
    const CODES: [&str; 57] = [
        "WrongPhase",
        "NoPassesLeft",
        "HandRequired",
        "NullNotAvailable",
        "NoRedealLeft",
        "AwaitingKontraAnswer",
        "NoKontra",
        "UnknownCard",
        "InvalidSlot",
        "NotEnoughCoins",
        "NoFreeSlot",
        "TargetRequired",
        "NoModAvailable",
        "UnknownContent",
        "InvalidStart",
        "GameWrongPhase",
        "NotYourTurn",
        "CardNotInHand",
        "MustFollowSuit",
        "InvalidDiscard",
        "InvalidDeal",
        "TrickInProgress",
        "InvalidDeclaration",
        "UnknownDeckCard",
        "DeckAtMinimum",
        "NothingToCleanse",
        "BlankCannotBeModified",
        "AlreadyModified",
        "CardCursed",
        "ModAtMaximum",
        "MissingCard",
        "GameNotAvailable",
        "OuvertRequired",
        "NoSkatSwapLeft",
        "NothingToTake",
        "InvalidSkatSwap",
        "RamschRequired",
        "DiscardForbidden",
        "NotLastCard",
        "AiToMove",
        "CannotChalk",
        "NotDecided",
        "NoRedealToUndo",
        "MustLeadDemanded",
        "InvalidLeadDemand",
        "NoLeadDemandLeft",
        "ConsumablesLocked",
        "LivesFull",
        "PainterPending",
        "RisePending",
        "InvalidRiseCards",
        "AlreadyUsed",
        "CurseStays",
        "ConditionNotMet",
        "NotInBossPub",
        "SchnappsRewardPending",
        "CardRewardPending",
    ];

    #[test]
    fn codes_keep_their_texts() {
        let texts: Vec<&str> = RunErrorCode::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(texts, CODES);
    }

    #[test]
    fn code_text_is_its_serialized_form() {
        for code in RunErrorCode::ALL {
            let json = serde_json::to_string(code).expect("serialize");
            assert_eq!(json, format!("\"{}\"", code.as_str()));
            let back: RunErrorCode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, *code);
        }
    }
}
