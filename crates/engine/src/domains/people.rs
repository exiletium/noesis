//! Famous people: birth and death dates and places, age, nationality,
//! occupation and what they are known for, with lists by birthplace,
//! first name and Nobel Prize, comparisons, and birthday look-ups.

use crate::engine::*;
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names (a leading ~ marks an alias that is too ambiguous to answer on its
/// own, such as a surname that is also a unit or a city) | born | birthplace |
/// died | place of death | nationality | occupation | known for.
/// Dates are YYYY-MM-DD or a bare year (negative for BCE), "c." when approximate.
const DATA: &str = "\
albert einstein,einstein|1879-03-14|Ulm, Germany|1955-04-18|Princeton, New Jersey, United States|German, Swiss, American|theoretical physicist|special and general relativity, E = mc², the photoelectric effect and Brownian motion; Nobel Prize in Physics (1921)
isaac newton,sir isaac newton,~newton|1643-01-04|Woolsthorpe, Lincolnshire, England|1727-03-31|Kensington, London, England|English|physicist and mathematician|laws of motion and universal gravitation, calculus, optics; Principia (1687)
galileo galilei,galileo|1564-02-15|Pisa, Italy|1642-01-08|Arcetri, Florence, Italy|Italian|astronomer and physicist|telescopic astronomy (moons of Jupiter, phases of Venus), laws of falling bodies, support for heliocentrism
michael faraday,~faraday|1791-09-22|Newington Butts, London, England|1867-08-25|Hampton Court, England|English|physicist and chemist|electromagnetic induction, the electric motor and dynamo, laws of electrolysis, diamagnetism, discovery of benzene
marie curie,marie skłodowska-curie,marie sklodowska curie,madame curie,~curie|1867-11-07|Warsaw, Poland|1934-07-04|Passy, Haute-Savoie, France|Polish, French|physicist and chemist|radioactivity, discovery of polonium and radium; Nobel Prize in Physics (1903) and Chemistry (1911), the first person to win two
pierre curie|1859-05-15|Paris, France|1906-04-19|Paris, France|French|physicist|radioactivity, piezoelectricity, Curie's law; Nobel Prize in Physics (1903)
irène joliot-curie,irene joliot-curie|1897-09-12|Paris, France|1956-03-17|Paris, France|French|chemist|artificial radioactivity; Nobel Prize in Chemistry (1935)
henri becquerel,~becquerel|1852-12-15|Paris, France|1908-08-25|Le Croisic, France|French|physicist|discovery of radioactivity; Nobel Prize in Physics (1903)
charles darwin,~darwin|1809-02-12|Shrewsbury, England|1882-04-19|Downe, Kent, England|English|naturalist|evolution by natural selection; On the Origin of Species (1859)
nikola tesla,~tesla|1856-07-10|Smiljan, Croatia (then Austrian Empire)|1943-01-07|New York City, United States|Serbian-American|inventor and electrical engineer|alternating-current induction motor and polyphase power, the Tesla coil
thomas edison,thomas alva edison,~edison|1847-02-11|Milan, Ohio, United States|1931-10-18|West Orange, New Jersey, United States|American|inventor and businessman|phonograph, practical incandescent light bulb, motion-picture camera; 1,093 US patents
stephen hawking,hawking|1942-01-08|Oxford, England|2018-03-14|Cambridge, England|English|theoretical physicist|Hawking radiation, singularity theorems; A Brief History of Time (1988)
richard feynman,feynman|1918-05-11|Queens, New York City, United States|1988-02-15|Los Angeles, United States|American|theoretical physicist|quantum electrodynamics, Feynman diagrams; Nobel Prize in Physics (1965)
niels bohr,~bohr|1885-10-07|Copenhagen, Denmark|1962-11-18|Copenhagen, Denmark|Danish|physicist|Bohr model of the atom, complementarity; Nobel Prize in Physics (1922)
max planck,~planck|1858-04-23|Kiel, Germany|1947-10-04|Göttingen, Germany|German|theoretical physicist|founding quantum theory (Planck's constant); Nobel Prize in Physics (1918)
werner heisenberg,heisenberg|1901-12-05|Würzburg, Germany|1976-02-01|Munich, Germany|German|theoretical physicist|matrix mechanics and the uncertainty principle; Nobel Prize in Physics (1932)
erwin schrödinger,erwin schrodinger,schrödinger,schrodinger|1887-08-12|Vienna, Austria|1961-01-04|Vienna, Austria|Austrian|theoretical physicist|the Schrödinger equation, Schrödinger's cat; Nobel Prize in Physics (1933)
paul dirac,~dirac|1902-08-08|Bristol, England|1984-10-20|Tallahassee, Florida, United States|English|theoretical physicist|the Dirac equation, prediction of antimatter; Nobel Prize in Physics (1933)
james clerk maxwell,~maxwell|1831-06-13|Edinburgh, Scotland|1879-11-05|Cambridge, England|Scottish|physicist|Maxwell's equations of electromagnetism, kinetic theory of gases, first colour photograph
johannes kepler,~kepler|1571-12-27|Weil der Stadt, Germany|1630-11-15|Regensburg, Germany|German|astronomer and mathematician|laws of planetary motion
nicolaus copernicus,copernicus|1473-02-19|Toruń, Poland|1543-05-24|Frombork, Poland|Polish|astronomer|heliocentric model of the Solar System; De revolutionibus (1543)
aristotle|-384|Stagira, Greece|-322|Chalcis, Euboea, Greece|Greek|philosopher|logic, biology, ethics, politics and metaphysics; tutor of Alexander the Great
archimedes|c. -287|Syracuse, Sicily|-212|Syracuse, Sicily|Greek|mathematician and engineer|Archimedes' principle of buoyancy, the lever, approximation of π, volume of a sphere
pythagoras|c. -570|Samos, Greece|c. -495|Metapontum, Italy|Greek|philosopher and mathematician|the Pythagorean theorem and the Pythagorean school
socrates|c. -470|Athens, Greece|-399|Athens, Greece|Greek|philosopher|the Socratic method; known through the dialogues of Plato
plato|c. -427|Athens, Greece|-348|Athens, Greece|Greek|philosopher|The Republic, theory of Forms; founded the Academy in Athens
confucius|-551|Qufu, Lu (China)|-479|Qufu, Lu (China)|Chinese|philosopher|Confucianism; the Analects
leonhard euler,~euler|1707-04-15|Basel, Switzerland|1783-09-18|Saint Petersburg, Russia|Swiss|mathematician|Euler's identity and formula, graph theory, notation f(x), e, i and π
carl friedrich gauss,~gauss|1777-04-30|Brunswick, Germany|1855-02-23|Göttingen, Germany|German|mathematician and physicist|number theory, the normal distribution, least squares, Gauss's law
alan turing,turing|1912-06-23|Maida Vale, London, England|1954-06-07|Wilmslow, Cheshire, England|English|mathematician and computer scientist|the Turing machine, breaking Enigma at Bletchley Park, the Turing test
ada lovelace,augusta ada king|1815-12-10|London, England|1852-11-27|London, England|English|mathematician|notes on Babbage's Analytical Engine with the first published computer algorithm
blaise pascal,~pascal|1623-06-19|Clermont-Ferrand, France|1662-08-19|Paris, France|French|mathematician, physicist and philosopher|Pascal's triangle, probability theory, Pascal's law of pressure, the Pensées
rené descartes,rene descartes,descartes|1596-03-31|La Haye en Touraine, France|1650-02-11|Stockholm, Sweden|French|philosopher and mathematician|\"I think, therefore I am\", Cartesian coordinates and analytic geometry
gottfried wilhelm leibniz,gottfried leibniz,leibniz|1646-07-01|Leipzig, Germany|1716-11-14|Hanover, Germany|German|mathematician and philosopher|calculus (independently of Newton), binary arithmetic, monadology
louis pasteur,pasteur|1822-12-27|Dole, France|1895-09-28|Marnes-la-Coquette, France|French|chemist and microbiologist|germ theory, pasteurization, vaccines for rabies and anthrax
gregor mendel,mendel|1822-07-20|Hynčice, Silesia (now Czech Republic)|1884-01-06|Brno, Moravia (now Czech Republic)|Austrian (Czech Silesian)|biologist and friar|laws of inheritance from pea-plant experiments; founder of genetics
dmitri mendeleev,mendeleev|1834-02-08|Tobolsk, Russia|1907-02-02|Saint Petersburg, Russia|Russian|chemist|the periodic table of the elements (1869)
antoine lavoisier,lavoisier|1743-08-26|Paris, France|1794-05-08|Paris, France|French|chemist|conservation of mass, naming oxygen and hydrogen; the father of modern chemistry
robert boyle,~boyle|1627-01-25|Lismore, Ireland|1691-12-31|London, England|Anglo-Irish|chemist and physicist|Boyle's law; The Sceptical Chymist (1661)
enrico fermi,~fermi|1901-09-29|Rome, Italy|1954-11-28|Chicago, United States|Italian-American|physicist|the first nuclear reactor (Chicago Pile-1), Fermi–Dirac statistics; Nobel Prize in Physics (1938)
j. robert oppenheimer,robert oppenheimer,oppenheimer|1904-04-22|New York City, United States|1967-02-18|Princeton, New Jersey, United States|American|theoretical physicist|director of the Los Alamos laboratory of the Manhattan Project
ernest rutherford,rutherford|1871-08-30|Brightwater, New Zealand|1937-10-19|Cambridge, England|New Zealander, British|physicist|the atomic nucleus (gold-foil experiment), alpha and beta radiation; Nobel Prize in Chemistry (1908)
rosalind franklin|1920-07-25|Notting Hill, London, England|1958-04-16|Chelsea, London, England|English|chemist and X-ray crystallographer|X-ray images of DNA (Photo 51) that revealed its double helix
francis crick|1916-06-08|Northampton, England|2004-07-28|San Diego, California, United States|English|molecular biologist|co-discovered the double-helix structure of DNA (1953); Nobel Prize in Physiology or Medicine (1962)
edwin hubble,~hubble|1889-11-20|Marshfield, Missouri, United States|1953-09-28|San Marino, California, United States|American|astronomer|galaxies beyond the Milky Way and the expansion of the universe (Hubble's law)
carl sagan,sagan|1934-11-09|Brooklyn, New York City, United States|1996-12-20|Seattle, Washington, United States|American|astronomer and science communicator|Cosmos (1980), Pale Blue Dot, the Voyager Golden Record
urbain le verrier,le verrier,urbain leverrier,leverrier|1811-03-11|Saint-Lô, France|1877-09-23|Paris, France|French|astronomer and mathematician|predicted the position of Neptune by calculation (1846)
johann gottfried galle,galle|1812-06-09|Radis, Saxony, Germany|1910-07-10|Potsdam, Germany|German|astronomer|first to observe Neptune knowingly, on 23 September 1846, using Le Verrier's prediction
john couch adams|1819-06-05|Laneast, Cornwall, England|1892-01-21|Cambridge, England|English|astronomer and mathematician|predicted Neptune's position independently of Le Verrier
william herschel,~herschel|1738-11-15|Hanover, Germany|1822-08-25|Slough, England|German-British|astronomer and composer|discovered Uranus (1781) and infrared radiation
alexander fleming|1881-08-06|Darvel, Scotland|1955-03-11|London, England|Scottish|microbiologist|discovery of penicillin (1928); Nobel Prize in Physiology or Medicine (1945)
sigmund freud,freud|1856-05-06|Příbor, Moravia (now Czech Republic)|1939-09-23|London, England|Austrian|neurologist|founder of psychoanalysis; The Interpretation of Dreams (1899)
florence nightingale|1820-05-12|Florence, Italy|1910-08-13|London, England|British|nurse and statistician|founder of modern nursing (Crimean War), polar-area diagrams
jane goodall|1934-04-03|London, England|2025-10-01|Los Angeles, California, United States|English|primatologist|decades-long study of wild chimpanzees at Gombe, Tanzania
leonardo da vinci,da vinci|1452-04-15|Vinci, Italy|1519-05-02|Amboise, France|Italian|painter, engineer and scientist|Mona Lisa, The Last Supper, Vitruvian Man, notebooks of inventions
michelangelo,michelangelo buonarroti|1475-03-06|Caprese, Italy|1564-02-18|Rome, Italy|Italian|sculptor, painter and architect|David, the Sistine Chapel ceiling, the dome of St Peter's Basilica
raphael,raffaello sanzio|1483|Urbino, Italy|1520-04-06|Rome, Italy|Italian|painter and architect|The School of Athens, the Sistine Madonna
vincent van gogh,van gogh|1853-03-30|Zundert, Netherlands|1890-07-29|Auvers-sur-Oise, France|Dutch|painter|The Starry Night, Sunflowers; about 2,100 artworks in a decade
pablo picasso,picasso|1881-10-25|Málaga, Spain|1973-04-08|Mougins, France|Spanish|painter and sculptor|co-founder of Cubism; Les Demoiselles d'Avignon, Guernica
claude monet,monet|1840-11-14|Paris, France|1926-12-05|Giverny, France|French|painter|founder of Impressionism; Impression, Sunrise, the Water Lilies series
rembrandt,rembrandt van rijn|1606-07-15|Leiden, Netherlands|1669-10-04|Amsterdam, Netherlands|Dutch|painter|The Night Watch, self-portraits, mastery of light and shadow
frida kahlo|1907-07-06|Coyoacán, Mexico City, Mexico|1954-07-13|Coyoacán, Mexico City, Mexico|Mexican|painter|self-portraits; The Two Fridas
salvador dalí,salvador dali,dalí,~dali|1904-05-11|Figueres, Spain|1989-01-23|Figueres, Spain|Spanish|painter|Surrealism; The Persistence of Memory
andy warhol,warhol|1928-08-06|Pittsburgh, Pennsylvania, United States|1987-02-22|New York City, United States|American|artist|pop art; Campbell's Soup Cans, Marilyn Diptych
william shakespeare,shakespeare|1564|Stratford-upon-Avon, England|1616-04-23|Stratford-upon-Avon, England|English|playwright and poet|Hamlet, Macbeth, Romeo and Juliet, King Lear, 154 sonnets
mary shelley,mary wollstonecraft shelley|1797-08-30|Somers Town, London, England|1851-02-01|Belgravia, London, England|English|novelist|Frankenstein; or, The Modern Prometheus (1818)
jane austen|1775-12-16|Steventon, Hampshire, England|1817-07-18|Winchester, England|English|novelist|Pride and Prejudice, Sense and Sensibility, Emma
charles dickens,dickens|1812-02-07|Portsmouth, England|1870-06-09|Higham, Kent, England|English|novelist|Oliver Twist, A Christmas Carol, Great Expectations, David Copperfield
mark twain,samuel clemens,samuel langhorne clemens|1835-11-30|Florida, Missouri, United States|1910-04-21|Redding, Connecticut, United States|American|writer and humorist|Adventures of Huckleberry Finn, The Adventures of Tom Sawyer
ernest hemingway,hemingway|1899-07-21|Oak Park, Illinois, United States|1961-07-02|Ketchum, Idaho, United States|American|novelist|The Old Man and the Sea, A Farewell to Arms; Nobel Prize in Literature (1954)
leo tolstoy,tolstoy|1828-09-09|Yasnaya Polyana, Russia|1910-11-20|Astapovo, Russia|Russian|novelist|War and Peace, Anna Karenina
fyodor dostoevsky,dostoevsky|1821-11-11|Moscow, Russia|1881-02-09|Saint Petersburg, Russia|Russian|novelist|Crime and Punishment, The Brothers Karamazov
edgar allan poe,~poe|1809-01-19|Boston, Massachusetts, United States|1849-10-07|Baltimore, Maryland, United States|American|writer and poet|The Raven, The Tell-Tale Heart; pioneer of the detective story
j. k. rowling,jk rowling,j.k. rowling,rowling|1965-07-31|Yate, Gloucestershire, England|||British|author|the Harry Potter series
j. r. r. tolkien,jrr tolkien,j.r.r. tolkien,tolkien|1892-01-03|Bloemfontein, South Africa|1973-09-02|Bournemouth, England|English|writer and philologist|The Hobbit, The Lord of the Rings
george orwell,orwell|1903-06-25|Motihari, India|1950-01-21|London, England|English|novelist and essayist|Nineteen Eighty-Four, Animal Farm
dante alighieri,dante|c. 1265|Florence, Italy|1321-09-14|Ravenna, Italy|Italian|poet|the Divine Comedy
johann wolfgang von goethe,goethe|1749-08-28|Frankfurt, Germany|1832-03-22|Weimar, Germany|German|poet, novelist and scientist|Faust, The Sorrows of Young Werther
victor hugo|1802-02-26|Besançon, France|1885-05-22|Paris, France|French|novelist and poet|Les Misérables, The Hunchback of Notre-Dame
oscar wilde|1854-10-16|Dublin, Ireland|1900-11-30|Paris, France|Irish|writer and playwright|The Picture of Dorian Gray, The Importance of Being Earnest
virginia woolf|1882-01-25|Kensington, London, England|1941-03-28|Rodmell, Sussex, England|English|novelist|Mrs Dalloway, To the Lighthouse, A Room of One's Own
emily dickinson|1830-12-10|Amherst, Massachusetts, United States|1886-05-15|Amherst, Massachusetts, United States|American|poet|nearly 1,800 poems, almost all published after her death
maya angelou|1928-04-04|St. Louis, Missouri, United States|2014-05-28|Winston-Salem, North Carolina, United States|American|poet and memoirist|I Know Why the Caged Bird Sings
toni morrison|1931-02-18|Lorain, Ohio, United States|2019-08-05|The Bronx, New York City, United States|American|novelist|Beloved, Song of Solomon; Nobel Prize in Literature (1993)
stephen king|1947-09-21|Portland, Maine, United States|||American|author|horror novels: Carrie, The Shining, It, The Stand
agatha christie|1890-09-15|Torquay, England|1976-01-12|Wallingford, England|English|crime novelist|Hercule Poirot and Miss Marple; the best-selling novelist of all time
hans christian andersen|1805-04-02|Odense, Denmark|1875-08-04|Copenhagen, Denmark|Danish|author|fairy tales: The Little Mermaid, The Ugly Duckling, The Snow Queen
walt whitman|1819-05-31|West Hills, New York, United States|1892-03-26|Camden, New Jersey, United States|American|poet|Leaves of Grass
truman capote|1924-09-30|New Orleans, Louisiana, United States|1984-08-25|Los Angeles, United States|American|writer|Breakfast at Tiffany's, In Cold Blood
albert camus,camus|1913-11-07|Mondovi, French Algeria|1960-01-04|Villeblevin, France|French|writer and philosopher|The Stranger, The Plague; Nobel Prize in Literature (1957)
jean-paul sartre,jean paul sartre,sartre|1905-06-21|Paris, France|1980-04-15|Paris, France|French|philosopher and writer|existentialism; Being and Nothingness; declined the Nobel Prize in Literature (1964)
wolfgang amadeus mozart,mozart|1756-01-27|Salzburg, Austria|1791-12-05|Vienna, Austria|Austrian|composer|The Magic Flute, Don Giovanni, the Requiem; over 600 works
ludwig van beethoven,beethoven|1770|Bonn, Germany|1827-03-26|Vienna, Austria|German|composer and pianist|nine symphonies (the Ninth with the Ode to Joy), Moonlight Sonata; composed while deaf
johann sebastian bach,~bach|1685-03-31|Eisenach, Germany|1750-07-28|Leipzig, Germany|German|composer|the Brandenburg Concertos, The Well-Tempered Clavier, the Mass in B minor
frédéric chopin,frederic chopin,chopin|1810-03-01|Żelazowa Wola, Poland|1849-10-17|Paris, France|Polish|composer and pianist|nocturnes, études, mazurkas and polonaises for piano
pyotr ilyich tchaikovsky,tchaikovsky|1840-05-07|Votkinsk, Russia|1893-11-06|Saint Petersburg, Russia|Russian|composer|Swan Lake, The Nutcracker, the 1812 Overture
elvis presley,elvis|1935-01-08|Tupelo, Mississippi, United States|1977-08-16|Memphis, Tennessee, United States|American|singer and actor|\"the King of Rock and Roll\"; Heartbreak Hotel, Hound Dog
john lennon|1940-10-09|Liverpool, England|1980-12-08|New York City, United States|English|musician|co-founder of the Beatles; Imagine
paul mccartney|1942-06-18|Liverpool, England|||English|musician|co-founder of the Beatles; Yesterday, Hey Jude
michael jackson|1958-08-29|Gary, Indiana, United States|2009-06-25|Los Angeles, United States|American|singer and dancer|\"the King of Pop\"; Thriller, the best-selling album of all time
marvin gaye|1939-04-02|Washington, D.C., United States|1984-04-01|Los Angeles, United States|American|singer and songwriter|What's Going On, Let's Get It On, Sexual Healing; Motown
aretha franklin|1942-03-25|Memphis, Tennessee, United States|2018-08-16|Detroit, Michigan, United States|American|singer|\"the Queen of Soul\"; Respect
stevie wonder|1950-05-13|Saginaw, Michigan, United States|||American|singer, songwriter and musician|Superstition, Songs in the Key of Life; 25 Grammy Awards
diana ross|1944-03-26|Detroit, Michigan, United States|||American|singer and actress|lead singer of the Supremes
smokey robinson|1940-02-19|Detroit, Michigan, United States|||American|singer and songwriter|the Miracles; Motown vice-president
madonna|1958-08-16|Bay City, Michigan, United States|||American|singer|\"the Queen of Pop\"; Like a Virgin, Vogue
eminem,marshall mathers|1972-10-17|St. Joseph, Missouri, United States|||American|rapper|The Marshall Mathers LP, Lose Yourself
louis armstrong|1901-08-04|New Orleans, Louisiana, United States|1971-07-06|Queens, New York City, United States|American|jazz trumpeter and singer|What a Wonderful World, Hello, Dolly!
fats domino|1928-02-26|New Orleans, Louisiana, United States|2017-10-24|Harvey, Louisiana, United States|American|pianist and singer|Blueberry Hill, Ain't That a Shame
lil wayne|1982-09-27|New Orleans, Louisiana, United States|||American|rapper|Tha Carter III
harry connick jr.,harry connick jr|1967-09-11|New Orleans, Louisiana, United States|||American|singer, pianist and actor|big-band jazz albums; When Harry Met Sally soundtrack
bob dylan|1941-05-24|Duluth, Minnesota, United States|||American|singer-songwriter|Like a Rolling Stone, Blowin' in the Wind; Nobel Prize in Literature (2016)
freddie mercury|1946-09-05|Stone Town, Zanzibar|1991-11-24|Kensington, London, England|British|singer and songwriter|lead singer of Queen; Bohemian Rhapsody
david bowie|1947-01-08|Brixton, London, England|2016-01-10|New York City, United States|English|singer and songwriter|Ziggy Stardust, Space Oddity, Heroes
prince,prince rogers nelson|1958-06-07|Minneapolis, Minnesota, United States|2016-04-21|Chanhassen, Minnesota, United States|American|singer, songwriter and multi-instrumentalist|Purple Rain, When Doves Cry
whitney houston|1963-08-09|Newark, New Jersey, United States|2012-02-11|Beverly Hills, California, United States|American|singer and actress|I Will Always Love You, The Bodyguard
taylor swift|1989-12-13|West Reading, Pennsylvania, United States|||American|singer-songwriter|Fearless, 1989, Folklore; the Eras Tour
beyoncé,beyonce,beyoncé knowles|1981-09-04|Houston, Texas, United States|||American|singer|Crazy in Love, Lemonade; the most Grammy Awards of any artist
john williams|1932-02-08|Floral Park, New York, United States|||American|composer and conductor|film scores for Star Wars, Jaws, E.T., Indiana Jones, Jurassic Park and Harry Potter
bob marley|1945-02-06|Nine Mile, Jamaica|1981-05-11|Miami, Florida, United States|Jamaican|singer-songwriter|reggae; No Woman, No Cry, One Love
jimi hendrix|1942-11-27|Seattle, Washington, United States|1970-09-18|London, England|American|guitarist and singer|Purple Haze, the Woodstock Star-Spangled Banner
kurt cobain|1967-02-20|Aberdeen, Washington, United States|1994-04-05|Seattle, Washington, United States|American|musician|lead singer of Nirvana; Smells Like Teen Spirit
frank sinatra|1915-12-12|Hoboken, New Jersey, United States|1998-05-14|Los Angeles, United States|American|singer and actor|My Way, Fly Me to the Moon, New York, New York
johnny cash|1932-02-26|Kingsland, Arkansas, United States|2003-09-12|Nashville, Tennessee, United States|American|singer-songwriter|I Walk the Line, Folsom Prison Blues
ray charles|1930-09-23|Albany, Georgia, United States|2004-06-10|Beverly Hills, California, United States|American|singer and pianist|Georgia on My Mind, Hit the Road Jack
jack white|1975-07-09|Detroit, Michigan, United States|||American|musician|the White Stripes; Seven Nation Army
alice cooper|1948-02-04|Detroit, Michigan, United States|||American|singer|shock rock; School's Out
tom hanks|1956-07-09|Concord, California, United States|||American|actor and filmmaker|Forrest Gump, Philadelphia, Saving Private Ryan, Cast Away; two Academy Awards for Best Actor
steven spielberg,spielberg|1946-12-18|Cincinnati, Ohio, United States|||American|film director and producer|Jaws, E.T., Jurassic Park, Schindler's List, Saving Private Ryan
betty white|1922-01-17|Oak Park, Illinois, United States|2021-12-31|Brentwood, Los Angeles, United States|American|actress and comedian|The Mary Tyler Moore Show, The Golden Girls; a television career over eight decades
robert pattinson|1986-05-13|London, England|||English|actor|Twilight, The Batman
marilyn monroe|1926-06-01|Los Angeles, United States|1962-08-04|Brentwood, Los Angeles, United States|American|actress and model|Gentlemen Prefer Blondes, Some Like It Hot
charlie chaplin|1889-04-16|Walworth, London, England|1977-12-25|Corsier-sur-Vevey, Switzerland|English|actor and filmmaker|the Tramp; The Gold Rush, Modern Times, The Great Dictator
alfred hitchcock,hitchcock|1899-08-13|Leytonstone, London, England|1980-04-29|Bel Air, Los Angeles, United States|English|film director|Psycho, Vertigo, Rear Window, The Birds
audrey hepburn|1929-05-04|Ixelles, Brussels, Belgium|1993-01-20|Tolochenaz, Switzerland|British|actress and humanitarian|Roman Holiday, Breakfast at Tiffany's
meryl streep|1949-06-22|Summit, New Jersey, United States|||American|actress|Kramer vs. Kramer, Sophie's Choice, The Iron Lady; 21 Academy Award nominations
leonardo dicaprio|1974-11-11|Los Angeles, United States|||American|actor|Titanic, Inception, The Revenant
brad pitt|1963-12-18|Shawnee, Oklahoma, United States|||American|actor|Fight Club, Once Upon a Time in Hollywood
morgan freeman|1937-06-01|Memphis, Tennessee, United States|||American|actor|The Shawshank Redemption, Million Dollar Baby, Driving Miss Daisy
denzel washington|1954-12-28|Mount Vernon, New York, United States|||American|actor|Glory, Training Day, Malcolm X
tom cruise|1962-07-03|Syracuse, New York, United States|||American|actor|Top Gun, the Mission: Impossible series
clint eastwood|1930-05-31|San Francisco, California, United States|||American|actor and director|Dirty Harry, Unforgiven, Million Dollar Baby
walt disney|1901-12-05|Chicago, Illinois, United States|1966-12-15|Burbank, California, United States|American|animator and entrepreneur|Mickey Mouse, Snow White and the Seven Dwarfs, Disneyland
robin williams|1951-07-21|Chicago, Illinois, United States|2014-08-11|Paradise Cay, California, United States|American|actor and comedian|Mrs. Doubtfire, Good Will Hunting, Dead Poets Society, Aladdin
oprah winfrey,oprah|1954-01-29|Kosciusko, Mississippi, United States|||American|talk-show host and media executive|The Oprah Winfrey Show (1986–2011)
martin scorsese,scorsese|1942-11-17|Queens, New York City, United States|||American|film director|Taxi Driver, Goodfellas, The Departed
stanley kubrick,kubrick|1928-07-26|Manhattan, New York City, United States|1999-03-07|Childwickbury, England|American|film director|2001: A Space Odyssey, A Clockwork Orange, The Shining
quentin tarantino,tarantino|1963-03-27|Knoxville, Tennessee, United States|||American|film director|Pulp Fiction, Kill Bill, Django Unchained
christopher nolan|1970-07-30|Westminster, London, England|||British-American|film director|The Dark Knight, Inception, Interstellar, Oppenheimer
francis ford coppola|1939-04-07|Detroit, Michigan, United States|||American|film director|The Godfather trilogy, Apocalypse Now
lily tomlin|1939-09-01|Detroit, Michigan, United States|||American|actress and comedian|Laugh-In, Nashville, Grace and Frankie
tom selleck|1945-01-29|Detroit, Michigan, United States|||American|actor|Magnum, P.I., Blue Bloods
grace kelly|1929-11-12|Philadelphia, Pennsylvania, United States|1982-09-14|Monaco|American|actress and Princess of Monaco|Rear Window, The Country Girl
judy garland|1922-06-10|Grand Rapids, Minnesota, United States|1969-06-22|London, England|American|actress and singer|Dorothy in The Wizard of Oz; Over the Rainbow
elizabeth taylor|1932-02-27|Hampstead, London, England|2011-03-23|Los Angeles, United States|British-American|actress|Cleopatra, Who's Afraid of Virginia Woolf?
james dean|1931-02-08|Marion, Indiana, United States|1955-09-30|Cholame, California, United States|American|actor|Rebel Without a Cause, East of Eden
bruce lee|1940-11-27|San Francisco, California, United States|1973-07-20|Hong Kong|Hong Kong-American|martial artist and actor|Enter the Dragon; founded Jeet Kune Do
jackie chan|1954-04-07|Hong Kong|||Hong Kong|actor and martial artist|Police Story, Rush Hour
arnold schwarzenegger,schwarzenegger|1947-07-30|Thal, Austria|||Austrian-American|actor, bodybuilder and politician|The Terminator; Governor of California (2003–2011)
ellen degeneres|1958-01-26|Metairie, Louisiana, United States|||American|comedian and talk-show host|The Ellen DeGeneres Show (2003–2022)
reese witherspoon|1976-03-22|New Orleans, Louisiana, United States|||American|actress and producer|Legally Blonde, Walk the Line
stephen colbert|1964-05-13|Washington, D.C., United States|||American|comedian and television host|The Colbert Report, The Late Show
stephen fry|1957-08-24|Hampstead, London, England|||English|actor, comedian and writer|A Bit of Fry & Laurie, QI, Jeeves and Wooster
stephen sondheim,sondheim|1930-03-22|New York City, United States|2021-11-26|Roxbury, Connecticut, United States|American|composer and lyricist|West Side Story (lyrics), Sweeney Todd, Into the Woods
stephen wolfram|1959-08-29|London, England|||British-American|computer scientist and physicist|Mathematica, Wolfram Alpha, A New Kind of Science
stephen curry|1988-03-14|Akron, Ohio, United States|||American|basketball player|Golden State Warriors; the NBA's all-time leader in three-pointers
sarah jessica parker|1965-03-25|Nelsonville, Ohio, United States|||American|actress|Carrie Bradshaw in Sex and the City
sarah bernhardt|1844-10-22|Paris, France|1923-03-26|Paris, France|French|stage actress|\"the Divine Sarah\"; La Dame aux Camélias, Hamlet
sarah palin|1964-02-11|Sandpoint, Idaho, United States|||American|politician|Governor of Alaska (2006–2009); 2008 Republican vice-presidential nominee
sarah silverman|1970-12-01|Bethesda, Maryland, United States|||American|comedian and actress|The Sarah Silverman Program
sarah vaughan|1924-03-27|Newark, New Jersey, United States|1990-04-03|Hidden Hills, California, United States|American|jazz singer|\"Sassy\"; Misty, Send in the Clowns
george washington,~washington|1732-02-22|Westmoreland County, Virginia, United States|1799-12-14|Mount Vernon, Virginia, United States|American|military officer and politician|1st President of the United States (1789–1797); commander of the Continental Army
abraham lincoln,~lincoln|1809-02-12|Hodgenville, Kentucky, United States|1865-04-15|Washington, D.C., United States|American|lawyer and politician|16th President of the United States (1861–1865); led the Union through the Civil War, Emancipation Proclamation, Gettysburg Address
thomas jefferson,jefferson|1743-04-13|Shadwell, Virginia, United States|1826-07-04|Charlottesville, Virginia, United States|American|politician and planter|principal author of the Declaration of Independence; 3rd President of the United States (1801–1809)
benjamin franklin,ben franklin|1706-01-17|Boston, Massachusetts, United States|1790-04-17|Philadelphia, Pennsylvania, United States|American|statesman, scientist and printer|lightning experiments and the lightning rod, bifocals; a Founding Father
john f. kennedy,john f kennedy,jfk,john kennedy|1917-05-29|Brookline, Massachusetts, United States|1963-11-22|Dallas, Texas, United States|American|politician|35th President of the United States (1961–1963); assassinated in Dallas
franklin d. roosevelt,franklin d roosevelt,franklin roosevelt,fdr|1882-01-30|Hyde Park, New York, United States|1945-04-12|Warm Springs, Georgia, United States|American|politician|32nd President of the United States (1933–1945); the New Deal, World War II
theodore roosevelt,teddy roosevelt|1858-10-27|New York City, United States|1919-01-06|Oyster Bay, New York, United States|American|politician and naturalist|26th President of the United States (1901–1909); Nobel Peace Prize (1906)
barack obama,obama,~barack|1961-08-04|Honolulu, Hawaii, United States|||American|lawyer and politician|44th President of the United States (2009–2017); Nobel Peace Prize (2009)
donald trump,trump|1946-06-14|Queens, New York City, United States|||American|businessman and politician|45th and 47th President of the United States (2017–2021, 2025–)
joe biden,joseph biden|1942-11-20|Scranton, Pennsylvania, United States|||American|politician|46th President of the United States (2021–2025)
ronald reagan,reagan|1911-02-06|Tampico, Illinois, United States|2004-06-05|Los Angeles, United States|American|actor and politician|40th President of the United States (1981–1989)
bill clinton|1946-08-19|Hope, Arkansas, United States|||American|politician|42nd President of the United States (1993–2001)
george w. bush,george w bush|1946-07-06|New Haven, Connecticut, United States|||American|politician|43rd President of the United States (2001–2009)
jimmy carter|1924-10-01|Plains, Georgia, United States|2024-12-29|Plains, Georgia, United States|American|politician|39th President of the United States (1977–1981); Nobel Peace Prize (2002)
richard nixon,nixon|1913-01-09|Yorba Linda, California, United States|1994-04-22|New York City, United States|American|politician|37th President of the United States (1969–1974); resigned after Watergate
winston churchill|1874-11-30|Blenheim Palace, Oxfordshire, England|1965-01-24|Kensington, London, England|British|statesman and writer|Prime Minister of the United Kingdom (1940–1945, 1951–1955); Nobel Prize in Literature (1953)
queen elizabeth ii,elizabeth ii|1926-04-21|Mayfair, London, England|2022-09-08|Balmoral Castle, Scotland|British|monarch|Queen of the United Kingdom (1952–2022), the longest reign of any British monarch
queen victoria|1819-05-24|Kensington Palace, London, England|1901-01-22|Osborne House, Isle of Wight, England|British|monarch|Queen of the United Kingdom (1837–1901) and Empress of India
napoleon,napoleon bonaparte,napoleon i|1769-08-15|Ajaccio, Corsica, France|1821-05-05|Longwood, Saint Helena|French|military leader and emperor|Emperor of the French (1804–1814, 1815); the Napoleonic Wars and Code
julius caesar|-100-07-12|Rome|-44-03-15|Rome|Roman|general and statesman|conquest of Gaul, crossing the Rubicon, dictator of Rome; assassinated on the Ides of March
cleopatra,cleopatra vii|-69|Alexandria, Egypt|-30|Alexandria, Egypt|Ptolemaic Egyptian|pharaoh|the last active ruler of Ptolemaic Egypt; allied with Julius Caesar and Mark Antony
alexander the great|-356|Pella, Macedonia|-323|Babylon|Macedonian Greek|king and conqueror|an empire from Greece to India by the age of 30
charlemagne,charles the great|c. 747|Francia|814-01-28|Aachen, Germany|Frankish|king and emperor|King of the Franks; first Holy Roman Emperor (crowned 800)
genghis khan|c. 1162|Delüün Boldog, Mongolia|1227|Western Xia (China)|Mongol|khan|founder of the Mongol Empire
martin luther king jr.,martin luther king jr,martin luther king,mlk,dr. martin luther king jr.|1929-01-15|Atlanta, Georgia, United States|1968-04-04|Memphis, Tennessee, United States|American|Baptist minister and civil-rights leader|the \"I Have a Dream\" speech, Montgomery bus boycott; Nobel Peace Prize (1964)
mahatma gandhi,gandhi,mohandas gandhi,mohandas karamchand gandhi|1869-10-02|Porbandar, India|1948-01-30|New Delhi, India|Indian|lawyer and independence leader|nonviolent resistance (satyagraha); led India's independence movement
mother teresa,saint teresa of calcutta|1910-08-26|Skopje, Ottoman Empire (now North Macedonia)|1997-09-05|Kolkata, India|Albanian-Indian|Catholic nun and missionary|Missionaries of Charity in Kolkata; Nobel Peace Prize (1979)
nelson mandela,mandela|1918-07-18|Mvezo, South Africa|2013-12-05|Johannesburg, South Africa|South African|anti-apartheid activist and politician|27 years in prison; first President of democratic South Africa (1994–1999); Nobel Peace Prize (1993)
harriet tubman|c. 1822|Dorchester County, Maryland, United States|1913-03-10|Auburn, New York, United States|American|abolitionist|escaped slavery and led about 70 people to freedom on the Underground Railroad; Union scout and spy
frederick douglass|c. 1818|Talbot County, Maryland, United States|1895-02-20|Washington, D.C., United States|American|abolitionist and writer|Narrative of the Life of Frederick Douglass (1845)
rosa parks|1913-02-04|Tuskegee, Alabama, United States|2005-10-24|Detroit, Michigan, United States|American|civil-rights activist|refused to give up her bus seat in Montgomery (1955)
malcolm x|1925-05-19|Omaha, Nebraska, United States|1965-02-21|Manhattan, New York City, United States|American|Muslim minister and human-rights activist|The Autobiography of Malcolm X
susan b. anthony,susan b anthony|1820-02-15|Adams, Massachusetts, United States|1906-03-13|Rochester, New York, United States|American|suffragist|campaign for women's right to vote
adolf hitler,hitler|1889-04-20|Braunau am Inn, Austria|1945-04-30|Berlin, Germany|Austrian-German|dictator|leader of Nazi Germany (1933–1945); started World War II in Europe and perpetrated the Holocaust
joseph stalin,stalin|1878-12-18|Gori, Georgia|1953-03-05|Kuntsevo Dacha, Moscow, Russia|Soviet (Georgian)|dictator|leader of the Soviet Union (1924–1953)
vladimir lenin,lenin|1870-04-22|Simbirsk, Russia|1924-01-21|Gorki, Russia|Russian|revolutionary and politician|led the Bolshevik Revolution (1917); founder of the Soviet Union
mao zedong,mao tse-tung,chairman mao|1893-12-26|Shaoshan, Hunan, China|1976-09-09|Beijing, China|Chinese|revolutionary and politician|founder of the People's Republic of China (1949)
karl marx,~marx|1818-05-05|Trier, Germany|1883-03-14|London, England|German|philosopher and economist|The Communist Manifesto (1848), Das Kapital
joan of arc,jeanne d'arc|c. 1412|Domrémy, France|1431-05-30|Rouen, France|French|military leader and saint|led French forces at the Siege of Orléans (1429); burned at the stake
christopher columbus,~columbus|1451|Genoa, Italy|1506-05-20|Valladolid, Spain|Genoese (Italian)|navigator and explorer|voyages across the Atlantic to the Americas (1492)
marco polo|1254|Venice, Italy|1324-01-08|Venice, Italy|Venetian|merchant and explorer|travels through Asia to the court of Kublai Khan
henry viii,henry the eighth,king henry viii|1491-06-28|Greenwich, England|1547-01-28|Palace of Whitehall, London, England|English|monarch|King of England (1509–1547); six wives, break with the Roman Catholic Church
elizabeth i,queen elizabeth i|1533-09-07|Greenwich, England|1603-03-24|Richmond Palace, Surrey, England|English|monarch|Queen of England (1558–1603); defeat of the Spanish Armada
louis xiv|1638-09-05|Saint-Germain-en-Laye, France|1715-09-01|Versailles, France|French|monarch|\"the Sun King\"; reigned 72 years (1643–1715), built Versailles
peter the great|1672-06-09|Moscow, Russia|1725-02-08|Saint Petersburg, Russia|Russian|tsar and emperor|modernized Russia; founded Saint Petersburg
catherine the great|1729-05-02|Stettin, Prussia (now Szczecin, Poland)|1796-11-17|Saint Petersburg, Russia|Russian (German-born)|empress|Empress of Russia (1762–1796)
otto von bismarck,bismarck|1815-04-01|Schönhausen, Prussia|1898-07-30|Friedrichsruh, Germany|German|statesman|unified Germany (1871); first Chancellor of the German Empire
simón bolívar,simon bolivar,bolívar,bolivar|1783-07-24|Caracas, Venezuela|1830-12-17|Santa Marta, Colombia|Venezuelan|military and political leader|led the independence of Venezuela, Colombia, Ecuador, Peru and Bolivia
che guevara,ernesto guevara|1928-06-14|Rosario, Argentina|1967-10-09|La Higuera, Bolivia|Argentine|Marxist revolutionary|a leader of the Cuban Revolution
fidel castro|1926-08-13|Birán, Cuba|2016-11-25|Havana, Cuba|Cuban|revolutionary and politician|leader of Cuba (1959–2008)
angela merkel,merkel|1954-07-17|Hamburg, Germany|||German|politician and physical chemist|Chancellor of Germany (2005–2021)
emmanuel macron,macron|1977-12-21|Amiens, France|||French|politician|President of France (2017–)
vladimir putin,putin|1952-10-07|Leningrad (Saint Petersburg), Russia|||Russian|politician|President of Russia (2000–2008, 2012–)
xi jinping|1953-06-15|Beijing, China|||Chinese|politician|General Secretary of the Chinese Communist Party (2012–) and President of China (2013–)
dalai lama,14th dalai lama,tenzin gyatso|1935-07-06|Taktser, Amdo, Tibet|||Tibetan|spiritual leader|the 14th Dalai Lama; Nobel Peace Prize (1989)
pope francis,jorge mario bergoglio|1936-12-17|Buenos Aires, Argentina|2025-04-21|Vatican City|Argentine|pope|head of the Catholic Church (2013–2025), the first pope from the Americas
martin luther|1483-11-10|Eisleben, Germany|1546-02-18|Eisleben, Germany|German|theologian|the Ninety-five Theses (1517) that began the Protestant Reformation
immanuel kant,kant|1724-04-22|Königsberg, Prussia|1804-02-12|Königsberg, Prussia|German|philosopher|Critique of Pure Reason, the categorical imperative
friedrich nietzsche,nietzsche|1844-10-15|Röcken, Prussia|1900-08-25|Weimar, Germany|German|philosopher|Thus Spoke Zarathustra, \"God is dead\", the Übermensch
john locke,~locke|1632-08-29|Wrington, Somerset, England|1704-10-28|High Laver, Essex, England|English|philosopher|Two Treatises of Government, empiricism; the father of liberalism
adam smith|1723-06-16|Kirkcaldy, Scotland|1790-07-17|Edinburgh, Scotland|Scottish|economist and philosopher|The Wealth of Nations (1776), the invisible hand
john maynard keynes,keynes|1883-06-05|Cambridge, England|1946-04-21|Tilton, Sussex, England|English|economist|Keynesian economics; The General Theory (1936)
noam chomsky,chomsky|1928-12-07|Philadelphia, Pennsylvania, United States|||American|linguist and philosopher|generative grammar, the Chomsky hierarchy
amelia earhart|1897-07-24|Atchison, Kansas, United States|1937-07-02|Pacific Ocean (disappeared near Howland Island)|American|aviator|first woman to fly solo across the Atlantic (1932)
neil armstrong|1930-08-05|Wapakoneta, Ohio, United States|2012-08-25|Cincinnati, Ohio, United States|American|astronaut|first person to walk on the Moon (Apollo 11, 20 July 1969)
buzz aldrin|1930-01-20|Glen Ridge, New Jersey, United States|||American|astronaut|second person to walk on the Moon (Apollo 11)
yuri gagarin,gagarin|1934-03-09|Klushino, Russia|1968-03-27|Kirzhach, Russia|Soviet|cosmonaut|first human in space (Vostok 1, 12 April 1961)
orville wright|1871-08-19|Dayton, Ohio, United States|1948-01-30|Dayton, Ohio, United States|American|aviation pioneer|with his brother Wilbur, the first powered, controlled aeroplane flight (17 December 1903)
wilbur wright|1867-04-16|Millville, Indiana, United States|1912-05-30|Dayton, Ohio, United States|American|aviation pioneer|with his brother Orville, the first powered, controlled aeroplane flight (17 December 1903)
charles lindbergh,lindbergh|1902-02-04|Detroit, Michigan, United States|1974-08-26|Kipahulu, Maui, Hawaii, United States|American|aviator|first solo non-stop transatlantic flight (Spirit of St. Louis, 1927)
henry ford|1863-07-30|Springwells Township, Michigan, United States|1947-04-07|Dearborn, Michigan, United States|American|industrialist|Ford Motor Company, the Model T and the moving assembly line
bill gates|1955-10-28|Seattle, Washington, United States|||American|businessman and philanthropist|co-founder of Microsoft; the Gates Foundation
steve jobs|1955-02-24|San Francisco, California, United States|2011-10-05|Palo Alto, California, United States|American|entrepreneur|co-founder of Apple; the Macintosh, iPod and iPhone
steve wozniak,woz|1950-08-11|San Jose, California, United States|||American|computer engineer|co-founder of Apple; designed the Apple I and Apple II
jeff bezos,bezos|1964-01-12|Albuquerque, New Mexico, United States|||American|businessman|founder of Amazon and Blue Origin
elon musk,musk|1971-06-28|Pretoria, South Africa|||South African-American|businessman|Tesla, SpaceX, Neuralink; owner of X (Twitter)
mark zuckerberg,zuckerberg|1984-05-14|White Plains, New York, United States|||American|businessman|co-founder of Facebook (Meta)
warren buffett,buffett|1930-08-30|Omaha, Nebraska, United States|||American|investor|chairman of Berkshire Hathaway; \"the Oracle of Omaha\"
larry ellison|1944-08-17|New York City, United States|||American|businessman|co-founder of Oracle Corporation
sergey brin|1973-08-21|Moscow, Russia|||American (Russian-born)|computer scientist and businessman|co-founder of Google
larry page|1973-03-26|East Lansing, Michigan, United States|||American|computer scientist and businessman|co-founder of Google; PageRank
tim berners-lee,tim berners lee|1955-06-08|London, England|||English|computer scientist|inventor of the World Wide Web (1989)
linus torvalds|1969-12-28|Helsinki, Finland|||Finnish-American|software engineer|creator of the Linux kernel and Git
andrew carnegie,carnegie|1835-11-25|Dunfermline, Scotland|1919-08-11|Lenox, Massachusetts, United States|Scottish-American|industrialist and philanthropist|Carnegie Steel; funded over 2,500 public libraries
john d. rockefeller,john d rockefeller,rockefeller|1839-07-08|Richford, New York, United States|1937-05-23|Ormond Beach, Florida, United States|American|businessman and philanthropist|founder of Standard Oil
derek jeter,jeter|1974-06-26|Pequannock Township, New Jersey, United States|||American|baseball player|shortstop for the New York Yankees (1995–2014); 5 World Series titles, 3,465 hits
albert pujols,pujols|1980-01-16|Santo Domingo, Dominican Republic|||Dominican-American|baseball player|first baseman (St. Louis Cardinals, Los Angeles Angels); 703 home runs
babe ruth|1895-02-06|Baltimore, Maryland, United States|1948-08-16|New York City, United States|American|baseball player|714 career home runs; New York Yankees
jackie robinson|1919-01-31|Cairo, Georgia, United States|1972-10-24|Stamford, Connecticut, United States|American|baseball player|broke Major League Baseball's colour line (Brooklyn Dodgers, 1947)
michael jordan|1963-02-17|Brooklyn, New York City, United States|||American|basketball player|6 NBA championships with the Chicago Bulls
lebron james|1984-12-30|Akron, Ohio, United States|||American|basketball player|the NBA's all-time leading scorer; 4 NBA championships
kobe bryant|1978-08-23|Philadelphia, Pennsylvania, United States|2020-01-26|Calabasas, California, United States|American|basketball player|5 NBA championships with the Los Angeles Lakers
muhammad ali,cassius clay|1942-01-17|Louisville, Kentucky, United States|2016-06-03|Scottsdale, Arizona, United States|American|boxer|three-time world heavyweight champion; \"float like a butterfly, sting like a bee\"
joe louis|1914-05-13|LaFayette, Alabama, United States|1981-04-12|Las Vegas, Nevada, United States|American|boxer|world heavyweight champion (1937–1949), 25 title defences
serena williams|1981-09-26|Saginaw, Michigan, United States|||American|tennis player|23 Grand Slam singles titles
roger federer,federer|1981-08-08|Basel, Switzerland|||Swiss|tennis player|20 Grand Slam singles titles, 8 at Wimbledon
usain bolt|1986-08-21|Sherwood Content, Jamaica|||Jamaican|sprinter|world records in the 100 m (9.58 s) and 200 m (19.19 s); 8 Olympic gold medals
michael phelps|1985-06-30|Baltimore, Maryland, United States|||American|swimmer|23 Olympic gold medals, the most of any athlete
pelé,pele|1940-10-23|Três Corações, Brazil|2022-12-29|São Paulo, Brazil|Brazilian|footballer|three World Cup titles (1958, 1962, 1970)
diego maradona,maradona|1960-10-30|Lanús, Argentina|2020-11-25|Tigre, Argentina|Argentine|footballer|1986 World Cup winner; the \"Hand of God\" goal
lionel messi,messi|1987-06-24|Rosario, Argentina|||Argentine|footballer|2022 World Cup winner; 8 Ballon d'Or awards
cristiano ronaldo|1985-02-05|Funchal, Madeira, Portugal|||Portuguese|footballer|the all-time top scorer in men's international football
tiger woods|1975-12-30|Cypress, California, United States|||American|golfer|15 major championships
wayne gretzky,gretzky|1961-01-26|Brantford, Ontario, Canada|||Canadian|ice hockey player|\"the Great One\"; the NHL's leading career point scorer
tom brady|1977-08-03|San Mateo, California, United States|||American|American football player|quarterback with 7 Super Bowl titles
simone biles|1997-03-14|Columbus, Ohio, United States|||American|gymnast|the most decorated gymnast in history
jesse owens|1913-09-12|Oakville, Alabama, United States|1980-03-31|Tucson, Arizona, United States|American|track and field athlete|4 gold medals at the 1936 Berlin Olympics
peyton manning|1976-03-24|New Orleans, Louisiana, United States|||American|American football player|quarterback; 2 Super Bowl titles, 5 MVP awards
";

struct Person {
    names: Vec<&'static str>,
    f: Vec<&'static str>,
}

fn people() -> &'static Vec<Person> {
    static P: OnceLock<Vec<Person>> = OnceLock::new();
    P.get_or_init(|| DATA.lines().map(|l| {
        let f: Vec<&str> = l.split('|').collect();
        Person { names: f[0].split(',').collect(), f }
    }).collect())
}

/// Whether the curated table knows this person (any alias).
pub(crate) fn known(name: &str) -> bool {
    find(name, false).is_some()
}

/// A person by name; `bare` excludes the ambiguous aliases marked with ~.
fn find(name: &str, bare: bool) -> Option<&'static Person> {
    let n = name.trim().trim_start_matches("the ").trim_end_matches("'s").trim_end_matches('’').trim().replace(", jr", " jr").replace(", sr", " sr");
    let n = n.as_str();
    people().iter().find(|p| p.names.iter().any(|x| *x == n || (!bare && x.strip_prefix('~') == Some(n))))
}

fn display(p: &Person) -> String {
    let n = p.names[0];
    let small = ["of", "the", "van", "von", "da", "de", "le", "jr.", "di"];
    n.split(' ').enumerate().map(|(i, w)| {
        if i > 0 && small.contains(&w) {
            if w == "jr." { "Jr.".to_string() } else { w.to_string() }
        } else if w.len() <= 3 && w.chars().all(|c| c.is_ascii_lowercase() || c == '.') && matches!(w, "ii" | "iii" | "iv" | "vi" | "vii" | "viii" | "xiv") {
            w.to_uppercase()
        } else {
            let mut c = w.chars();
            let first: String = c.next().map(|f| f.to_uppercase().collect()).unwrap_or_default();
            // hyphenated and dotted parts: Joliot-Curie, J. R. R.
            let rest: String = c.as_str().to_string();
            let joined = first + &rest;
            joined.split('-').map(|part| {
                let mut c = part.chars();
                c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
            }).collect::<Vec<_>>().join("-")
        }
    }).collect::<Vec<_>>().join(" ")
        .replace("Mcc", "McC").replace("Dicaprio", "DiCaprio").replace("Degeneres", "DeGeneres").replace("Le Verrier", "Le Verrier")
}

/// (year, month, day, approximate)
fn parse_date(s: &str) -> Option<(i32, Option<(u32, u32)>, bool)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (approx, s) = match s.strip_prefix("c. ") { Some(r) => (true, r), None => (false, s) };
    let neg = s.starts_with('-');
    let body = s.trim_start_matches('-');
    let mut it = body.split('-');
    let y: i32 = it.next()?.parse().ok()?;
    let y = if neg { -y } else { y };
    let md = match (it.next(), it.next()) {
        (Some(m), Some(d)) => Some((m.parse().ok()?, d.parse().ok()?)),
        _ => None,
    };
    Some((y, md, approx))
}

fn year_text(y: i32) -> String {
    if y < 0 { format!("{} BCE", -y) } else if y < 1000 { format!("{y} CE") } else { y.to_string() }
}

/// Just the year, e.g. "c. 747", "814", "100 BCE".
fn years_of(s: &str) -> String {
    match parse_date(s) {
        Some((y, _, approx)) => format!("{}{}", if approx { "c. " } else { "" }, if y < 0 { format!("{} BCE", -y) } else { y.to_string() }),
        None => String::new(),
    }
}

fn date_text(s: &str) -> String {
    match parse_date(s) {
        Some((y, Some((m, d)), _)) => match NaiveDate::from_ymd_opt(y, m, d) {
            Some(dt) if y >= 1000 => dt.format("%A, %B %-d, %Y").to_string(),
            _ => format!("{} {d}, {}", ["", "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][m as usize], year_text(y)),
        },
        Some((y, None, approx)) => format!("{}{}", if approx { "c. " } else { "" }, year_text(y)),
        None => String::new(),
    }
}

/// Age in whole years at `to` (or today), or an approximate span.
fn age(p: &Person) -> Option<String> {
    let (by, bmd, bapprox) = parse_date(p.f[1])?;
    let (ty, tmd, tapprox) = match parse_date(p.f[3]) {
        Some(d) => d,
        None => {
            let t = super::misc::today();
            (t.year(), Some((t.month(), t.day())), false)
        }
    };
    // there is no year 0 between 1 BCE and 1 CE
    let span = ty - by - if by < 0 && ty > 0 { 1 } else { 0 };
    let years = match (bmd, tmd) {
        (Some(b), Some(t)) => span - if t < b { 1 } else { 0 },
        _ => return Some(format!("about {} years", span)),
    };
    Some(format!("{}{years} years", if bapprox || tapprox { "about " } else { "" }))
}

fn living(p: &Person) -> bool {
    p.f[3].is_empty()
}

fn card(p: &Person) -> Vec<Vec<Content>> {
    let mut v = vec![
        vec![text("full name"), text(display(p))],
        vec![text("born"), text(format!("{} in {}", date_text(p.f[1]), p.f[2]))],
    ];
    if living(p) {
        if let Some(a) = age(p) {
            v.push(vec![text("age"), text(a)]);
        }
    } else {
        v.push(vec![text("died"), text(format!("{} in {}", date_text(p.f[3]), p.f[4]))]);
        if let Some(a) = age(p) {
            v.push(vec![text("age at death"), text(a)]);
        }
    }
    v.push(vec![text("nationality"), text(p.f[5])]);
    v.push(vec![text("occupation"), text(p.f[6])]);
    v.push(vec![text("known for"), text(p.f[7])]);
    v
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn split_names(s: &str) -> Vec<&str> {
    // "Martin Luther King, Jr., Mother Teresa" keeps "Jr." with its name
    let mut out: Vec<&str> = Vec::new();
    let mut start = 0;
    let parts: Vec<(usize, &str)> = s.match_indices(", ").collect();
    let mut cuts = Vec::new();
    for (i, _) in parts {
        let after = &s[i + 2..];
        if after.starts_with("jr") || after.starts_with("sr") {
            continue;
        }
        cuts.push(i);
    }
    for c in cuts {
        out.push(&s[start..c]);
        start = c + 2;
    }
    out.push(&s[start..]);
    out.into_iter().flat_map(|x| re!(r"\s+(?:vs\.?|versus|and|&)\s+").split(x).collect::<Vec<_>>()).map(|x| x.trim().trim_start_matches("and ")).filter(|x| !x.is_empty()).collect()
}

fn list_answer(title: &str, people: Vec<&Person>) -> Option<Answer> {
    if people.is_empty() {
        return None;
    }
    let rows: Vec<Vec<Content>> = people.iter().map(|p| vec![text(display(p)), text(date_text(p.f[1])), text(p.f[6])]).collect();
    Some(Answer::new("people", vec![
        input(title.to_string()),
        primary(pod("Result", "People", text(people.iter().map(|p| display(p)).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Details", table(Some(vec!["name", "born", "occupation"]), rows)),
        pod("Note", "Note", text("among the famous people Noesis knows; not a complete list")),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).to_string();
    let l = re!(r"\s+").replace_all(&l, " ").replace('’', "'");

    // lists: "famous people from Detroit", "notable people born in New Orleans metro area"
    if let Some(m) = re!(r"^(?:famous|notable|well-known|celebrities|famous people|notable people)(?: people| persons)? (?:from|born in|of) (?:the )?(.+?)(?: metro(?:politan)? area| area)?$").captures(&l) {
        let place = m[1].trim();
        let ps: Vec<&Person> = people().iter().filter(|p| {
            let bp = p.f[2].to_lowercase();
            let first = bp.split(',').next().unwrap_or("").trim().to_string();
            first == place || bp.split(", ").any(|x| x == place) || (place.len() > 3 && bp.contains(&format!("{place},")))
        }).collect();
        // "new orleans metro area" also takes the suburbs
        let mut ps = ps;
        if place == "new orleans" {
            ps.extend(people().iter().filter(|p| p.f[2].starts_with("Metairie") || p.f[2].starts_with("Kenner")));
        }
        return list_answer(&format!("famous people born in {}", place), ps);
    }
    if let Some(m) = re!(r"^(?:famous|notable|well-known)? ?people (?:named|called|with the first name|with the name) ([a-z]+)$").captures(&l) {
        let n = &m[1];
        let alt = match n { "stephen" => Some("steven"), "steven" => Some("stephen"), "sara" => Some("sarah"), _ => None };
        let ps: Vec<&Person> = people().iter().filter(|p| { let f = p.names[0].split(' ').next().unwrap_or(""); f == n || Some(f) == alt }).collect();
        return list_answer(&format!("famous people named {}", display_word(n)), ps);
    }
    if let Some(m) = re!(r"^nobel (?:prize )?(?:winners|laureates|prize winners) (?:from|of|in) (?:the )?(.+)$").captures(&l) {
        let adj = match m[1].trim() { "france" => "french", "germany" => "german", "united states" | "us" | "usa" | "america" => "american", "united kingdom" | "uk" | "britain" | "england" => "english|british|scottish", "poland" => "polish", "denmark" => "danish", "austria" => "austrian", "italy" => "italian", "india" => "indian", "south africa" => "south african", x => x };
        let ps: Vec<&Person> = people().iter().filter(|p| p.f[7].contains("Nobel") && adj.split('|').any(|a| p.f[5].to_lowercase().contains(a))).collect();
        return list_answer(&format!("Nobel Prize winners from {}", display_word(m[1].trim())), ps);
    }

    // "X birthday moon phase"
    if let Some(m) = re!(r"^(.+?)(?:'s)? (?:birthday|birth date|date of birth) moon phase$|^moon phase (?:on|of) (.+?)(?:'s)? (?:birthday|birth date)$").captures(&l) {
        let p = find(m.get(1).or(m.get(2))?.as_str(), false)?;
        let (y, md, _) = parse_date(p.f[1])?;
        let (mo, d) = md?;
        let date = NaiveDate::from_ymd_opt(y, mo, d)?;
        let r = crate::engine::query(&format!("moon phase on {}", date.format("%B %-d, %Y")));
        let mut pods = vec![input(format!("{} | birthday ({}) | moon phase", display(p), date_text(p.f[1])))];
        pods.extend(r.pods.into_iter().filter(|x| x.id != "Input"));
        return Some(Answer::new("moon phase", pods));
    }

    // properties
    let prop = re!(r"^(?:(?:when was|when is) (.+?) (born|birthday)|(?:when did) (.+?) die|(?:where was) (.+?) born|(?:where did) (.+?) die|how old (?:is|was) (.+?)(?: when (?:he|she|they) died)?|(?:(birthday|birth date|date of birth|birthplace|place of birth|death date|date of death|place of death|age|age at death|nationality|occupation|net worth) (?:of )?(.+))|(.+?)(?:'s)? (birthday|birthdays|birth date|date of birth|birthplace|place of birth|death date|date of death|place of death|age|age at death|nationality|occupation|net worth|death))$").captures(&l);
    if let Some(m) = prop {
        let (what, who) = if let Some(w) = m.get(1) { ("birthday", w.as_str()) }
            else if let Some(w) = m.get(3) { ("death date", w.as_str()) }
            else if let Some(w) = m.get(4) { ("birthplace", w.as_str()) }
            else if let Some(w) = m.get(5) { ("place of death", w.as_str()) }
            else if let Some(w) = m.get(6) { ("age", w.as_str()) }
            else if let Some(k) = m.get(7) { (k.as_str(), m.get(8)?.as_str()) }
            else { (m.get(10)?.as_str(), m.get(9)?.as_str()) };
        let names = split_names(who);
        let ps: Option<Vec<&Person>> = names.iter().map(|n| find(n, false)).collect();
        if let Some(ps) = ps {
            let val = |p: &Person| -> String {
                match what {
                    "birthday" | "birthdays" | "birth date" | "date of birth" => date_text(p.f[1]),
                    "birthplace" | "place of birth" => p.f[2].to_string(),
                    "death date" | "date of death" | "death" => if living(p) { "living".into() } else { date_text(p.f[3]) },
                    "place of death" => if living(p) { "living".into() } else { p.f[4].to_string() },
                    "age" | "age at death" => {
                        let a = age(p).unwrap_or_default();
                        if living(p) { a } else { format!("{a} (died {})", date_text(p.f[3])) }
                    }
                    "nationality" => p.f[5].to_string(),
                    "occupation" => p.f[6].to_string(),
                    _ => "net worth is not tracked offline (it changes daily)".to_string(),
                }
            };
            let main = if ps.len() == 1 { val(ps[0]) } else { ps.iter().map(|p| format!("{}: {}", display(p), val(p))).collect::<Vec<_>>().join(" | ") };
            let mut pods = vec![
                input(format!("{} | {what}", ps.iter().map(|p| display(p)).collect::<Vec<_>>().join(", "))),
                primary(pod("Result", "Result", text(main))),
            ];
            if ps.len() == 1 {
                pods.push(pod("Properties", "Basic information", table(None, card(ps[0]))));
            }
            return Some(Answer::new("person", pods));
        }
    }

    // cards: "Albert Einstein", "facts about Galileo", "why was Albert Einstein famous", "who was Harriet Tubman"
    let (body, intro) = match re!(r"^(?:who (?:was|is|were|are) |facts about |tell me about |biography of |why (?:was|is) (.+?) famous$|what (?:was|is) (.+?) famous for$|(.+?)'s (?:scientific )?contributions$|(.+?) biography$)").captures(&l) {
        Some(m) => match m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4)) {
            Some(x) => (x.as_str().to_string(), true),
            None => (l[m.get(0)?.end()..].to_string(), true),
        },
        None => (l.to_string(), false),
    };
    let names = split_names(&body);
    let found: Option<Vec<&Person>> = names.iter().map(|n| find(n, !intro && names.len() == 1)).collect();
    let Some(ps) = found else {
        // a single person outside the curated table: the Wikidata extract,
        // for names of at least two words unless asked about explicitly
        if names.len() != 1 || (!intro && !names[0].contains(' ')) {
            return None;
        }
        let (name, about, card) = super::facts::person_card(names[0])?;
        if about.is_empty() {
            return None;
        }
        return Some(Answer::new("person", vec![
            input(format!("{name} | person")),
            primary(pod("Result", "Description", text(about))),
            card,
            pod("Source", "Source", text("Wikidata (CC0)")),
        ]));
    };
    if ps.len() == 1 {
        let p = ps[0];
        let headline = if l.contains("famous") || l.contains("contribution") { p.f[7].to_string() } else {
            format!("{} ({}–{}); {}", p.f[6], years_of(p.f[1]), if living(p) { String::new() } else { years_of(p.f[3]) }, p.f[7])
        };
        return Some(Answer::new("person", vec![
            input(format!("{} | person", display(p))),
            primary(pod("Result", if l.contains("famous") || l.contains("contribution") { "Known for" } else { "Summary" }, text(headline))),
            pod("Properties", "Basic information", table(None, card(p))),
        ]));
    }
    let mut header = vec![""];
    let tnames: Vec<String> = ps.iter().map(|p| display(p)).collect();
    header.extend(tnames.iter().map(|s| s.as_str()));
    let labels = ["born", "birthplace", "died", "age", "nationality", "occupation", "known for"];
    let rows: Vec<Vec<Content>> = labels.iter().map(|lab| {
        let mut row = vec![text(*lab)];
        for p in &ps {
            row.push(text(match *lab {
                "born" => date_text(p.f[1]),
                "birthplace" => p.f[2].to_string(),
                "died" => if living(p) { "living".to_string() } else { date_text(p.f[3]) },
                "age" => age(p).map(|a| if living(p) { a } else { format!("{a} at death") }).unwrap_or_default(),
                "nationality" => p.f[5].to_string(),
                "occupation" => p.f[6].to_string(),
                _ => p.f[7].to_string(),
            }));
        }
        row
    }).collect();
    Some(Answer::new("people", vec![
        input(format!("{} | comparison", tnames.join(", "))),
        primary(pod("Result", "Comparison", table(Some(header), rows))),
    ]))
}

fn display_word(s: &str) -> String {
    s.split(' ').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        for p in people() {
            assert_eq!(p.f.len(), 8, "{:?}", p.f);
            assert!(parse_date(p.f[1]).is_some(), "{:?}", p.f);
            assert!(p.f[3].is_empty() || parse_date(p.f[3]).is_some(), "{:?}", p.f);
            if let Some((y, Some((m, d)), _)) = parse_date(p.f[1]) {
                assert!(y < 0 || NaiveDate::from_ymd_opt(y, m, d).is_some(), "{:?}", p.f);
            }
        }
        let mut seen = std::collections::HashSet::new();
        for p in people() {
            for n in &p.names {
                assert!(seen.insert(n.trim_start_matches('~')), "duplicate name {n}");
            }
        }
    }
}
