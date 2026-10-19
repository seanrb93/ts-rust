//! The set-up of the doors, links and portals of each story level.

use crate::*;

/// Set up the doors (closed, locked or open), the links of the pads and the portals of the story level `lvlevelnum` (levels 2 to 27) by the difficulty (+76 of the game parameters);
/// the state of the story (`story_state1`) starts at 0 (@ 0x229928).
pub fn lvInitStoryState() {
    let offset = lw(g::lvlevelnum) - 2;
    sw(g::story_state1, 0);
    if (offset as u32) >= 26 {
        return;
    }
    match offset {
        0 => {
            doorClose(1);
            doorLock(1);
            linkDisable(padGetLinkNum(1324, 1127));
            bgPortalDisableFromRooms(18, 21);
            return;
        }
        1 | 2 | 4 | 5 | 8 | 9 | 10 | 11 | 13 | 14 | 15 | 16 | 17 | 18 | 20 | 23 | 24 => {
            return;
        }
        3 => {
            let params = g::curgameparms;
            let mut difficulty = lw(params + 76);
            if ((difficulty - 1) as u32) < 2 {
            linkDisable(padGetLinkNum(1244, 1245));
                difficulty = lw(params + 76);
            }
            if difficulty == 2 {
            linkDisable(padGetLinkNum(1212, 1396));
            }
        }
        6 => {
            doorClose(1);
            doorClose(2);
            doorClose(3);
            doorClose(4);
            doorClose(5);
            doorClose(6);
            doorClose(7);
            doorClose(8);
            doorClose(9);
            doorClose(10);
            doorClose(11);
            doorClose(12);
            doorClose(13);
            doorLock(1);
            doorLock(2);
            doorLock(3);
            doorLock(4);
            doorLock(5);
            doorLock(6);
            doorLock(7);
            doorLock(8);
            doorLock(9);
            doorLock(10);
            doorLock(11);
            doorLock(12);
            doorLock(13);
            linkDisable(padGetLinkNum(1098, 1102));
            linkDisable(padGetLinkNum(1081, 1008));
            linkDisable(padGetLinkNum(1012, 1037));
            linkDisable(padGetLinkNum(1314, 1037));
            linkDisable(padGetLinkNum(1031, 1037));
            linkDisable(padGetLinkNum(1193, 1194));
            linkDisable(padGetLinkNum(1185, 1186));
            linkDisable(padGetLinkNum(1128, 1388));
            linkDisable(padGetLinkNum(1152, 1154));
            bgPortalDisableFromRooms(92, 93);
            bgPortalDisableFromRooms(90, 69);
            bgPortalDisableFromRooms(71, 95);
            bgPortalDisableFromRooms(70, 95);
            bgPortalDisable(91);
            bgPortalDisable(92);
            bgPortalDisable(93);
            bgPortalDisableFromRooms(78, 69);
            bgPortalDisableFromRooms(3, 18);
            bgPortalDisableFromRooms(5, 13);
            bgPortalDisableFromRooms(13, 14);
            bgPortalDisableFromRooms(86, 98);
            bgPortalDisableFromRooms(37, 31);
            if ((lw(g::curgameparms + 76) - 1) as u32) >= 2 {
                return;
            }
            doorClose(14);
            doorLock(14);
            return;
        }
        7 => {}
        12 => {
            doorOpen(1);
            doorLock(1);
            doorClose(2);
            doorLock(2);
            linkDisable(padGetLinkNum(1141, 1381));
            doorOpen(3);
            doorLock(3);
            doorClose(4);
            doorLock(4);
            linkDisable(padGetLinkNum(1237, 1152));
            return;
        }
        19 => {
            doorClose(2);
            doorLock(2);
            linkDisable(padGetLinkNum(1042, 1044));
            bgPortalDisableFromRooms(30, 33);
            doorClose(1);
            doorLock(1);
            linkDisable(padGetLinkNum(1143, 1050));
            bgPortalDisableFromRooms(36, 35);
            return;
        }
        21 => {
            doorClose(2);
            doorLock(2);
            linkDisable(padGetLinkNum(1035, 1238));
            doorClose(1);
            doorLock(1);
            linkDisable(padGetLinkNum(1029, 1428));
            let params = g::curgameparms;
            let mut difficulty = lw(params + 76);
            if difficulty == 0 {
                doorClose(3);
                doorLock(3);
                linkDisable(padGetLinkNum(1112, 1116));
                doorClose(4);
                doorLock(4);
                linkDisable(padGetLinkNum(1113, 1236));
                difficulty = lw(params + 76);
            }
            let mut rest = difficulty - 1;
            if difficulty == 1 {
                doorClose(5);
                doorLock(5);
                linkDisable(padGetLinkNum(1214, 1218));
                rest = lw(params + 76) - 1;
            }
            if (rest as u32) >= 2 {
                return;
            }
            doorClose(6);
            doorLock(6);
            linkDisable(padGetLinkNum(1169, 1505));
            return;
        }
        22 => {
            doorClose(1);
            doorLock(1);
            bgPortalDisableFromRooms(71, 15);
            linkDisable(padGetLinkNum(1173, 1101));
            doorClose(2);
            doorLock(2);
            bgPortalDisableFromRooms(48, 12);
            linkDisable(padGetLinkNum(1194, 1182));
            doorClose(3);
            doorLock(3);
            bgPortalDisableFromRooms(37, 75);
            linkDisable(padGetLinkNum(1157, 1118));
            doorClose(5);
            doorLock(5);
            bgPortalDisableFromRooms(72, 31);
            linkDisable(padGetLinkNum(1165, 1088));
            linkDisable(padGetLinkNum(1308, 1428));
            linkDisable(padGetLinkNum(1308, 1156));
            linkDisable(padGetLinkNum(1309, 1428));
            linkDisable(padGetLinkNum(1309, 1155));
            linkDisable(padGetLinkNum(1445, 1248));
            linkDisable(padGetLinkNum(1210, 1245));
            linkDisable(padGetLinkNum(1298, 1245));
            linkDisable(padGetLinkNum(1298, 1246));
            return;
        }
        25 => {
            doorClose(5);
            doorLock(5);
            doorClose(6);
            doorLock(6);
            linkDisable(padGetLinkNum(1742, 1018));
            doorClose(7);
            doorLock(7);
            doorClose(8);
            doorLock(8);
            linkDisable(padGetLinkNum(1029, 1030));
            doorClose(1);
            doorLock(1);
            doorClose(2);
            doorLock(2);
            linkDisable(padGetLinkNum(1065, 1067));
            doorClose(3);
            doorLock(3);
            doorClose(4);
            doorLock(4);
            linkDisable(padGetLinkNum(1070, 1570));
            linkDisable(padGetLinkNum(1067, 1070));
            return;
        }
        _ => unreachable!(),
    }
    linkDisable(padGetLinkNum(1067, 1070));
}
