CREATE OR REPLACE FUNCTION create_all_table() RETURNS void
    LANGUAGE plpgsql
AS
$$

BEGIN
    DROP TABLE IF EXISTS player CASCADE;
    DROP TABLE IF EXISTS hero CASCADE;
    DROP TABLE IF EXISTS characters;
    DROP TABLE IF EXISTS army CASCADE;
    DROP TABLE IF EXISTS army_to_hero;
    DROP TABLE IF EXISTS skills CASCADE;
    DROP TABLE IF EXISTS speciallity CASCADE;

    CREATE TABLE player
    (
        id   serial PRIMARY KEY,
        name varchar(100) UNIQUE NOT NULL
    );

    CREATE TABLE skills
    (
        id       serial PRIMARY KEY,
        name     varchar UNIQUE NOT NULL,
        describe varchar        NOT NULL,
        effect   varchar        NOT NULL
    );

    CREATE TABLE speciallity
    (
        id   serial PRIMARY KEY,
        name varchar UNIQUE NOT NULL
    );

    CREATE TABLE characters
    (
        id         serial PRIMARY KEY,
        name       varchar(100) UNIQUE NOT NULL,
        side       varchar(100)        NOT NULL,
        class      varchar(100)        NOT NULL,
        offence    int                 NOT NULL,
        deffence   int                 NOT NULL,
        mana       int                 NOT NULL,
        knowledge  int                 NOT NULL,
        speciality int REFERENCES speciallity (id),
        portrait   bytea
    );

--     CREATE TABLE army
--     (
--         id       serial PRIMARY KEY,
--         name     varchar(100) UNIQUE NOT NULL,
--         level    int                 NOT NULL,
--         offence  int                 NOT NULL,
--         deffence int                 NOT NULL,
--         shots    int                 NOT NULL,
--         damage   varchar(100)        NOT NULL,
--         vital    int                 NOT NULL,
--         speed    int                 NOT NULL
--     );

    CREATE TABLE hero
    (
        id              serial PRIMARY KEY,
        characters      int REFERENCES characters (id),
        level           int NOT NULL,
        experience      int NOT NULL,
        offence         int NOT NULL,
        defence         int NOT NULL,
        mana            int NOT NULL,
        knowledge       int NOT NULL,
        specialityimage bytea,
        player          int REFERENCES player (id),
        skills          int REFERENCES skills (id)
    );

--     CREATE TABLE army_to_hero
--     (
--         id    serial PRIMARY KEY,
--         army  int REFERENCES army (id),
--         hero  int REFERENCES hero (id),
--         amout int NOT NULL,
--         slot  int NOT NULL
--     );
END;
$$;

SELECT create_all_table();

CREATE OR REPLACE FUNCTION delete_player(playerid int) RETURNS bool
    LANGUAGE plpgsql
AS
$$

BEGIN
    DELETE FROM player WHERE id = playerid;
    RETURN TRUE;
END;
$$;

CREATE OR REPLACE FUNCTION create_player(playername varchar) RETURNS int
    LANGUAGE plpgsql
AS
$$

BEGIN
    INSERT INTO player VALUES (default, playername);
    RETURN (SELECT id FROM player WHERE name = playername);
END;
$$;

CREATE OR REPLACE FUNCTION create_characters(name varchar, side varchar, class varchar, offence int, deffence int,
                                             mana int, knowledge int, speciality_id int) RETURNS int
    LANGUAGE plpgsql
AS
$$

BEGIN
    INSERT INTO characters VALUES (default, name, side, class, offence, deffence, mana, knowledge, speciality_id);
    RETURN (SELECT id FROM characters ORDER BY id DESC LIMIT 1);
END;
$$;

CREATE OR REPLACE FUNCTION delete_characters_by_name(namechar varchar) RETURNS void
    LANGUAGE plpgsql
AS
$$
BEGIN
    DELETE FROM characters WHERE characters.name = namechar;
END;
$$;

CREATE OR REPLACE FUNCTION delete_characters(charactersid int) RETURNS void
    LANGUAGE plpgsql
AS
$$
BEGIN
    DELETE FROM characters WHERE characters.id = charactersid;
END;
$$;

CREATE OR REPLACE FUNCTION create_hero(player int, characters_id int) RETURNS int
    LANGUAGE plpgsql
AS
$$
DECLARE
    charact characters%rowtype;
BEGIN
    SELECT * INTO charact FROM characters WHERE characters.id = characters_id;
    INSERT INTO hero
    VALUES (default, characters_id, 0, 0, charact.offence, charact.deffence, charact.mana, charact.knowledge, NULL,
            player);
    RETURN (SELECT id FROM hero ORDER BY id DESC LIMIT 1);
END;
$$;

CREATE OR REPLACE FUNCTION delete_hero(heroid int) RETURNS void
    LANGUAGE plpgsql
AS
$$
BEGIN
    DELETE FROM hero WHERE id = heroid;
END;
$$;

-- CREATE OR REPLACE FUNCTION add_unit_to_hero(playerid int, heroid int, slot int, unitid int, amount int) RETURNS void
--     LANGUAGE plpgsql
-- AS
-- $$
-- BEGIN
--     INSERT INTO army_to_hero VALUES (default, unitid, heroid, amount, slot);
-- END;
-- $$;
--
--
-- CREATE OR REPLACE FUNCTION remove_unit_to_hero(playerid int, heroid int, slot int) RETURNS void
--     LANGUAGE plpgsql
-- AS
-- $$
-- BEGIN
--     DELETE FROM army_to_hero WHERE player = playerid AND hero = heroid AND army_to_hero.slot = slot;
-- END;
-- $$;

CREATE OR REPLACE FUNCTION get_heroes(playerid int)
    RETURNS table
            (
                id           int,
                level        int,
                experience   int,
                offence      int,
                defence      int,
                mana         int,
                knowledge    int,
                name         varchar,
                side         varchar,
                class        varchar
--                 slot         int,
--                 amount       int,
--                 unit_name    varchar,
--                 unit_level   int,
--                 unit_offence int,
--                 unit_defence int,
--                 unit_shots   int,
--                 unit_damage  varchar,
--                 unit_vital   int,
--                 unit_speed   int
            )
    LANGUAGE plpgsql
AS
$$
BEGIN
    RETURN QUERY (SELECT hero.id,
                         hero.level,
                         hero.experience,
                         hero.offence,
                         hero.defence,
                         hero.mana,
                         hero.knowledge,
                         characters.name,
                         characters.side,
                         characters.class
--                          army_to_hero.slot,
--                          army_to_hero.amout,
--                          army.name,
--                          army.level,
--                          army.offence,
--                          army.deffence,
--                          army.shots,
--                          army.damage,
--                          army.vital,
--                          army.speed
                  FROM hero
                           JOIN characters ON hero.characters = characters.id
--                            JOIN army_to_hero ON army_to_hero.hero = hero.id
--                            JOIN army ON army_to_hero.army = army.id
                  WHERE hero.player = playerid
        ORDER BY hero.id);
END;
$$;

CREATE OR REPLACE FUNCTION get_players()
    RETURNS table
            (
                id   int,
                name varchar
            )
    LANGUAGE plpgsql
AS
$$
BEGIN
    RETURN QUERY (SELECT player.id, player.name FROM player);
END;
$$;

CREATE OR REPLACE FUNCTION set_level() RETURNS trigger AS
$$
DECLARE
    newlevel   int := 1;
    currentexp int;
BEGIN
    currentexp := old.experience;
    CASE
        WHEN currentexp < 1000 THEN newlevel := 2;
        WHEN currentexp >= 1000 AND currentexp < 2000 THEN newlevel := 2;
        WHEN currentexp >= 2000 AND currentexp < 3000 THEN newlevel := 3;
        WHEN currentexp >= 3000 AND currentexp < 4000 THEN newlevel := 4;
        ELSE newlevel := 5;
        END CASE;
    UPDATE hero SET level = newlevel WHERE id = new.id;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER thero
    AFTER UPDATE OF experience
    ON hero
    FOR EACH ROW
EXECUTE PROCEDURE set_level();

CREATE OR REPLACE FUNCTION modify_hero(
                nid            int,
                nexperience    int,
                noffence       int,
                ndefence       int,
                nmana          int,
                nknowledge     int
)
    RETURNS void
AS
$$
BEGIN
    UPDATE hero as h SET experience = nexperience, offence = noffence, defence = ndefence, mana = nmana, knowledge = nknowledge WHERE id = nid;
END;
$$ LANGUAGE plpgsql;

-- CREATE OR REPLACE FUNCTION create_unit(name varchar, level int, offence int, deffence int, shots int, damage varchar,
--                                        vital int, speed int) RETURNS int
--     LANGUAGE plpgsql
-- AS
-- $$
-- BEGIN
--     INSERT INTO army VALUES (default, name, level, offence, deffence, shots, damage, vital, speed);
--     RETURN (SELECT id FROM army ORDER BY id DESC LIMIT 1);
-- END;
-- $$;

CREATE OR REPLACE FUNCTION create_skill(name varchar, describe varchar, effect varchar) RETURNS int
    LANGUAGE plpgsql
AS
$$

BEGIN
    INSERT INTO skills VALUES (default, name, describe, effect);
    RETURN (SELECT id FROM skills ORDER BY id DESC LIMIT 1);
END;
$$;

CREATE OR REPLACE FUNCTION create_speciallity(name varchar) RETURNS int
    LANGUAGE plpgsql
AS
$$

BEGIN
    INSERT INTO speciallity VALUES (default, name);
    RETURN (SELECT id FROM speciallity ORDER BY id DESC LIMIT 1);
END;
$$;


CREATE OR REPLACE FUNCTION fill_tables() RETURNS void
    LANGUAGE plpgsql
AS
$$

BEGIN
    PERFORM create_skill('Доспехи', 'Уменьшает повреждение, наносимое отрядам героя на', '0,05');
    PERFORM create_skill('Интеллект', 'Увеличивает нормальный максимум баллов заклинаний героя на ', '0,25');
    PERFORM create_skill('Лидерство', 'Поднимает боевой дух отряда героя на 	', '1');
    PERFORM create_skill('Логистика', 'увеличивает баллы перемещения героя по суше на ', '0,1');
    PERFORM create_skill('Магия Воды', 'Позволяет герою творить заклинания Воды	', 'по уменьшенной цене');
    PERFORM create_skill('Магия Воздуха', 'Позволяет герою творить заклинания Воздуха', 'по уменьшенной цене');
    PERFORM create_skill('Магия Земли', 'Позволяет герою творить заклинания Земли', 'по уменьшенной цене');
    PERFORM create_skill('Магия Огня', 'Позволяет герою творить заклинания Огня	', 'по уменьшенной цене');
    PERFORM create_skill('Мудрость', 'Позволяет герою изучать заклинания	', '3 уровня');
    PERFORM create_skill('Нападение', 'Увеличивает весь урон, наносимый отрядами героя, на', '0,1');
    PERFORM create_skill('Тактика', 'Позволяет реорганизовать свои отряды прямо перед боем',
                         'в 3 рядах клеток от героя');
    PERFORM create_skill('Удача', 'Увеличивает удачу героя	', '1');
    PERFORM create_skill('Нектомантия', 'Позволяет восстановить из мертвых в виде Скелетов до',
                         '0,1 существ, убитых в бою');

--     PERFORM create_unit('Гремлин', 1, 3, 3, -1, '1-2', 4, 4);
--     PERFORM create_unit('Маг', 1, 11, 8, 24, '7-9', 25, 5);
--     PERFORM create_unit('Живой мертвец', 1, 5, 5, -1, '2-3', 15, 3);
--     PERFORM create_unit('Скелет	', 1, 5, 4, -1, '1-3', 6, 4);
--     PERFORM create_unit('Скелет-воин', 2, 6, 6, -1, '1-3', 6, 5);
--     PERFORM create_unit('Вампир', 1, 10, 9, -1, '5-8', 30, 6);

    PERFORM create_speciallity('Temp');

    PERFORM create_characters('Теодор', 'Башня', 'Маг', 0, 0, 3, 2, 1);
    PERFORM create_characters('Галтран', 'Некрополис', 'Рыцарь Смерти', 1, 2, 1, 2, 1);
    PERFORM create_characters('Тант', 'Некрополис', 'Некромант', 1, 0, 2, 2, 1);
    PERFORM create_characters('Сандро', 'Некрополис', 'Некромант', 1, 0, 2, 2, 1);
    PERFORM create_characters('Клавиус', 'Некрополис', 'Рыцарь Смерти', 1, 2, 1, 2, 1);

    PERFORM create_player('Алекс');
    PERFORM create_player('Айдан');
    PERFORM create_player('Айзек');
    PERFORM create_player('Джон');
    PERFORM create_player('Рекс');


    PERFORM create_hero(1, 1);
    PERFORM create_hero(2, 2);
    PERFORM create_hero(3, 3);
    PERFORM create_hero(4, 4);
    PERFORM create_hero(5, 5);


--     PERFORM add_unit_to_hero(1, 1, 1, 1, 13);
--     PERFORM add_unit_to_hero(1, 1, 2, 2, 5);
--     PERFORM add_unit_to_hero(2, 2, 1, 3, 37);
--     PERFORM add_unit_to_hero(3, 3, 1, 3, 30);
--     PERFORM add_unit_to_hero(3, 3, 2, 5, 60);
--     PERFORM add_unit_to_hero(3, 3, 3, 5, 50);
--     PERFORM add_unit_to_hero(4, 4, 1, 3, 30);
--     PERFORM add_unit_to_hero(5, 5, 1, 3, 98);

END;
$$;

SELECT fill_tables();