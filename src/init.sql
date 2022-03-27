DROP TABLE IF EXISTS heroes;
DROP TABLE IF EXISTS players;
DROP TABLE IF EXISTS character;
DROP TABLE IF EXISTS specialty;
DROP TABLE IF EXISTS classes;

-- Tables
CREATE TABLE players
(
    id   serial PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE classes
(
    id   serial PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE character
(
    id             serial PRIMARY KEY,
    name           text                        NOT NULL,
    class          int REFERENCES classes (id) NOT NULL,
    portrait_small bytea                       NOT NULL
);

CREATE TABLE specialty
(
    id    serial PRIMARY KEY,
    name  text  NOT NULL,
    image bytea NOT NULL,
    class int   NOT NULL REFERENCES classes (id)
);

CREATE TABLE heroes
(
    id             serial PRIMARY KEY,
    player_id      serial REFERENCES players (id) ON DELETE CASCADE,
    character_id   serial REFERENCES character (id),
    mana_current   int    NOT NULL CHECK (mana_current <= mana_max),
    mana_max       int    NOT NULL,
    xp             int    NOT NULL,
    luck           int    NOT NULL,
    morale         int    NOT NULL,
    specialty      int    NOT NULL REFERENCES specialty (id) ON DELETE CASCADE,
    primary_skills int[4] NOT NULL,
    level          int    NOT NULL
);

-- Initial data
CREATE OR REPLACE FUNCTION clear_all_tables() RETURNS void AS
$$
DELETE FROM heroes;
DELETE FROM players;
DELETE FROM character;
DELETE FROM specialty;
DELETE FROM classes;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION initial_data() RETURNS void AS
$$
SELECT clear_all_tables();
INSERT INTO players
VALUES (DEFAULT, 'Player1');

INSERT INTO classes
VALUES (DEFAULT, 'Class1');

INSERT INTO character
VALUES (DEFAULT, 'Character1', (SELECT id FROM classes), DECODE(
        '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',
        'base64'));

INSERT INTO specialty
VALUES (DEFAULT, 'Spec1', DECODE(
        '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',
        'base64'), (SELECT id FROM classes));


INSERT INTO heroes
VALUES (DEFAULT, (SELECT id FROM players), (SELECT MAX(id) FROM character), 10, 34, 10, 1, 1, (SELECT id FROM specialty), '{10, 11, 12, 13}', 1);
$$ LANGUAGE sql;

-- Functions
CREATE OR REPLACE FUNCTION set_hero_mana(hero_id int, new_current int, new_max int)
    RETURNS void AS
$$
UPDATE heroes
SET mana_current = new_current,
    mana_max     = new_max
WHERE id = hero_id;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION set_hero_xp(hero_id int, value int)
    RETURNS void AS
$$
UPDATE heroes
SET xp = value
WHERE id = hero_id;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION set_hero_morale(hero_id int, value int)
    RETURNS void AS
$$
UPDATE heroes
SET morale = value
WHERE id = hero_id;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION set_hero_luck(hero_id int, value int)
    RETURNS void AS
$$
UPDATE heroes
SET luck = value
WHERE id = hero_id;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION set_hero_pskill(hero_id int, primary_skill_id int, value int)
    RETURNS void AS
$$
UPDATE heroes
SET primary_skills[primary_skill_id] = value
WHERE id = hero_id;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION get_player_heroes(requested_player_id int)
    RETURNS table
            (
                id             int,
                portrait_small bytea
            )
AS
$$
SELECT h.id, c.portrait_small
FROM heroes AS h
         JOIN character c ON c.id = h.character_id
WHERE h.player_id = requested_player_id;
$$ LANGUAGE sql;

DROP FUNCTION get_hero;
CREATE OR REPLACE FUNCTION get_hero(requested_hero_id int)
    RETURNS table
            (
                name            text,
                portrait_small  bytea,
                class_name      text,
                mana_current    int,
                mana_max        int,
                xp              int,
                luck            int,
                morale          int,
                primary_skills  int[4],
                specialty_name  text,
                specialty_image bytea,
                level           int
            )
AS
$$
SELECT c.name,
       c.portrait_small,
       cl.name,
       h.mana_current,
       h.mana_max,
       h.xp,
       h.luck,
       h.morale,
       h.primary_skills,
       sp.name,
       sp.image,
       h.level
FROM heroes AS h
         JOIN character c ON c.id = h.character_id
         JOIN classes cl ON cl.id = c.class
         JOIN specialty sp ON cl.id = sp.class
WHERE h.id = requested_hero_id;
$$ LANGUAGE sql;
SELECT *
FROM
    get_hero(
            2);

CREATE OR REPLACE FUNCTION get_players()
    RETURNS table
            (
                player_id   int,
                player_name text
            )
AS
$$
SELECT id, name
FROM players;
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION setlevel() RETURNS trigger AS
$$
DECLARE
    newlevel   int := 1;
    currentexp int;
BEGIN
    currentexp := old.xp;
    CASE
        WHEN currentexp < 1000 THEN newlevel := 1;
        WHEN currentexp BETWEEN 1000 AND 2000 THEN newlevel := 2;
        WHEN currentexp BETWEEN 2000 AND 3000 THEN newlevel := 3;
        WHEN currentexp BETWEEN 3000 AND 4000 THEN newlevel := 4;
        ELSE newlevel := 5;
        END CASE;
    UPDATE heroes SET level = newlevel WHERE id = new.id;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

UPDATE heroes
SET level = 1,
    xp    = 1
WHERE id = 2;
SELECT level, xp
FROM heroes;

CREATE TRIGGER t_heroe
    AFTER UPDATE OF xp
    ON heroes
    FOR EACH ROW
EXECUTE PROCEDURE setlevel();

CREATE OR REPLACE FUNCTION clear_heroes() RETURNS void AS
$$
DELETE FROM heroes;
$$ LANGUAGE sql;