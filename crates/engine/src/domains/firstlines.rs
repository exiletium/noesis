//! Opening lines of famous books (public-domain texts and translations).
//!
//! "opening phrase of A Tale of Two Cities", "first line of Moby Dick",
//! "how does Pride and Prejudice begin".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// titles | display | author, year | opening
const LINES: &[(&[&str], &str, &str, &str)] = &[
    (&["a tale of two cities", "tale of two cities"], "A Tale of Two Cities", "Charles Dickens, 1859", "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, …"),
    (&["moby dick", "moby-dick", "moby-dick; or, the whale"], "Moby-Dick", "Herman Melville, 1851", "Call me Ishmael."),
    (&["pride and prejudice"], "Pride and Prejudice", "Jane Austen, 1813", "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife."),
    (&["anna karenina"], "Anna Karenina", "Leo Tolstoy, 1878 (tr. Constance Garnett)", "Happy families are all alike; every unhappy family is unhappy in its own way."),
    (&["a christmas carol", "christmas carol"], "A Christmas Carol", "Charles Dickens, 1843", "Marley was dead: to begin with."),
    (&["david copperfield"], "David Copperfield", "Charles Dickens, 1850", "Whether I shall turn out to be the hero of my own life, or whether that station will be held by anybody else, these pages must show."),
    (&["great expectations"], "Great Expectations", "Charles Dickens, 1861", "My father's family name being Pirrip, and my Christian name Philip, my infant tongue could make of both names nothing longer or more explicit than Pip."),
    (&["the metamorphosis", "metamorphosis"], "The Metamorphosis", "Franz Kafka, 1915 (tr. David Wyllie)", "One morning, when Gregor Samsa woke from troubled dreams, he found himself transformed in his bed into a horrible vermin."),
    (&["jane eyre"], "Jane Eyre", "Charlotte Brontë, 1847", "There was no possibility of taking a walk that day."),
    (&["frankenstein"], "Frankenstein", "Mary Shelley, 1818", "You will rejoice to hear that no disaster has accompanied the commencement of an enterprise which you have regarded with such evil forebodings."),
    (&["adventures of huckleberry finn", "the adventures of huckleberry finn", "huckleberry finn", "huck finn"], "Adventures of Huckleberry Finn", "Mark Twain, 1884", "You don't know about me without you have read a book by the name of The Adventures of Tom Sawyer; but that ain't no matter."),
    (&["the great gatsby", "great gatsby"], "The Great Gatsby", "F. Scott Fitzgerald, 1925", "In my younger and more vulnerable years my father gave me some advice that I've been turning over in my mind ever since."),
    (&["don quixote"], "Don Quixote", "Miguel de Cervantes, 1605 (tr. John Ormsby)", "In a village of La Mancha, the name of which I have no desire to call to mind, there lived not long since one of those gentlemen that keep a lance in the lance-rack, an old buckler, a lean hack, and a greyhound for coursing."),
    (&["genesis", "the book of genesis", "book of genesis", "the bible", "bible"], "Genesis", "King James Version, 1611", "In the beginning God created the heaven and the earth."),
    (&["peter pan", "peter and wendy"], "Peter and Wendy", "J. M. Barrie, 1911", "All children, except one, grow up."),
    (&["alice's adventures in wonderland", "alice in wonderland", "alices adventures in wonderland"], "Alice's Adventures in Wonderland", "Lewis Carroll, 1865", "Alice was beginning to get very tired of sitting by her sister on the bank, and of having nothing to do: …"),
    (&["war and peace"], "War and Peace", "Leo Tolstoy, 1869 (tr. Louise and Aylmer Maude)", "\"Well, Prince, so Genoa and Lucca are now just family estates of the Buonapartes.\""),
    (&["the call of the wild", "call of the wild"], "The Call of the Wild", "Jack London, 1903", "Buck did not read the newspapers, or he would have known that trouble was brewing, not alone for himself, but for every tide-water dog, strong of muscle and with warm, long hair, from Puget Sound to San Diego."),
    (&["ulysses"], "Ulysses", "James Joyce, 1922", "Stately, plump Buck Mulligan came from the stairhead, bearing a bowl of lather on which a mirror and a razor lay crossed."),
    (&["mrs dalloway", "mrs. dalloway"], "Mrs Dalloway", "Virginia Woolf, 1925", "Mrs. Dalloway said she would buy the flowers herself."),
    (&["the scarlet letter", "scarlet letter"], "The Scarlet Letter", "Nathaniel Hawthorne, 1850", "A throng of bearded men, in sad-coloured garments and grey steeple-crowned hats, intermixed with women, some wearing hoods, and others bareheaded, was assembled in front of a wooden edifice, the door of which was heavily timbered with oak, and studded with iron spikes."),
    (&["the adventures of tom sawyer", "adventures of tom sawyer", "tom sawyer"], "The Adventures of Tom Sawyer", "Mark Twain, 1876", "\"Tom!\""),
    (&["dracula"], "Dracula", "Bram Stoker, 1897", "3 May. Bistritz.—Left Munich at 8:35 P. M., on 1st May, arriving at Vienna early next morning; should have arrived at 6:46, but train was an hour late."),
    (&["the picture of dorian gray", "picture of dorian gray"], "The Picture of Dorian Gray", "Oscar Wilde, 1890", "The studio was filled with the rich odour of roses, and when the light summer wind stirred amidst the trees of the garden, there came through the open door the heavy scent of the lilac, or the more delicate perfume of the pink-flowering thorn."),
    (&["the odyssey", "odyssey"], "The Odyssey", "Homer (tr. Samuel Butler, 1900)", "Tell me, O muse, of that ingenious hero who travelled far and wide after he had sacked the famous town of Troy."),
    (&["the iliad", "iliad"], "The Iliad", "Homer (tr. Samuel Butler, 1898)", "Sing, O goddess, the anger of Achilles son of Peleus, that brought countless ills upon the Achaeans."),
    (&["wuthering heights"], "Wuthering Heights", "Emily Brontë, 1847", "1801.—I have just returned from a visit to my landlord—the solitary neighbour that I shall be troubled with."),
    (&["little women"], "Little Women", "Louisa May Alcott, 1868", "\"Christmas won't be Christmas without any presents,\" grumbled Jo, lying on the rug."),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let m = re!(r"^(?:what is |what's |what are )?(?:the )?(?:first (?:line|lines|sentence|words)|opening (?:line|lines|phrase|sentence|words)|beginning|start) (?:of|in) (?:the (?:book|novel) )?(.+)$|^how does (?:the (?:book|novel) )?(.+?) (?:begin|start|open)$|^(.+?) (?:first line|opening line|opening sentence|opening words|first sentence)$").captures(&l)?;
    let title = m.get(1).or(m.get(2)).or(m.get(3))?.as_str().trim();
    let w = LINES.iter().find(|w| w.0.contains(&title))?;
    Some(Answer::new(format!("{} | opening", w.1), vec![
        pod("Input", "Input interpretation", text(format!("{} ({}) | opening line", w.1, w.2))),
        primary(pod("Result", "Opening line", text(w.3))),
    ]))
}
