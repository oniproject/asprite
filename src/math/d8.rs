/**
* Implements Dihedral Group D_8,
* see http://mathworld.wolfram.com/DihedralGroupD4.html,
* D8 is the same but with diagonals. Used for texture rotations.
*
* Vector xX(i), xY(i) is U-axis of sprite with rotation i
* Vector yY(i), yY(i) is V-axis of sprite with rotation i
* Rotations: 0 grad (0), 90 grad (2), 180 grad (4), 270 grad (6)
* Mirrors: vertical (8), main diagonal (10), horizontal (12), reverse diagonal (14)
* This is the small part of gameofbombs.com portal system. It works.
*/

type I = i8;
type F = f32;

// just for testing
const UX: [I; 16] = [1,  1,  0, -1, -1, -1,  0,  1,  1,  1, 0, -1, -1, -1,  0,  1];
const UY: [I; 16] = [0,  1,  1,  1,  0, -1, -1, -1,  0,  1, 1,  1,  0, -1, -1, -1];
const VX: [I; 16] = [0, -1, -1, -1,  0,  1,  1,  1,  0,  1, 1,  1,  0, -1, -1, -1];
const VY: [I; 16] = [1,  1,  0, -1, -1, -1,  0,  1, -1, -1, 0,  1,  1,  1,  0, -1];

const UX_F: [F; 16] = [1.0,  1.0,  0.0, -1.0, -1.0, -1.0,  0.0,  1.0,  1.0,  1.0, 0.0, -1.0, -1.0, -1.0,  0.0,  1.0];
const UY_F: [F; 16] = [0.0,  1.0,  1.0,  1.0,  0.0, -1.0, -1.0, -1.0,  0.0,  1.0, 1.0,  1.0,  0.0, -1.0, -1.0, -1.0];
const VX_F: [F; 16] = [0.0, -1.0, -1.0, -1.0,  0.0,  1.0,  1.0,  1.0,  0.0,  1.0, 1.0,  1.0,  0.0, -1.0, -1.0, -1.0];
const VY_F: [F; 16] = [1.0,  1.0,  0.0, -1.0, -1.0, -1.0,  0.0,  1.0, -1.0, -1.0, 0.0,  1.0,  1.0,  1.0,  0.0, -1.0];
//const tempMatrices = [];

const ROTATOR: [[I; 16]; 16] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [1, 2, 3, 4, 5, 6, 7, 0, 9, 10, 11, 12, 13, 14, 15, 8],
    [2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12, 13, 14, 15, 8, 9],
    [3, 4, 5, 6, 7, 0, 1, 2, 11, 12, 13, 14, 15, 8, 9, 10],
    [4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11],
    [5, 6, 7, 0, 1, 2, 3, 4, 13, 14, 15, 8, 9, 10, 11, 12],
    [6, 7, 0, 1, 2, 3, 4, 5, 14, 15, 8, 9, 10, 11, 12, 13],
    [7, 0, 1, 2, 3, 4, 5, 6, 15, 8, 9, 10, 11, 12, 13, 14],
    [8, 15, 14, 13, 12, 11, 10, 9, 0, 7, 6, 5, 4, 3, 2, 1],
    [9, 8, 15, 14, 13, 12, 11, 10, 1, 0, 7, 6, 5, 4, 3, 2],
    [10, 9, 8, 15, 14, 13, 12, 11, 2, 1, 0, 7, 6, 5, 4, 3],
    [11, 10, 9, 8, 15, 14, 13, 12, 3, 2, 1, 0, 7, 6, 5, 4],
    [12, 11, 10, 9, 8, 15, 14, 13, 4, 3, 2, 1, 0, 7, 6, 5],
    [13, 12, 11, 10, 9, 8, 15, 14, 5, 4, 3, 2, 1, 0, 7, 6],
    [14, 13, 12, 11, 10, 9, 8, 15, 6, 5, 4, 3, 2, 1, 0, 7],
    [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct D8(I);

pub const D8_E:  D8 = D8(0); //   0 grad
pub const D8_SE: D8 = D8(1);
pub const D8_S:  D8 = D8(2); //  90 grad
pub const D8_SW: D8 = D8(3);
pub const D8_W:  D8 = D8(4); // 180 grad
pub const D8_NW: D8 = D8(5);
pub const D8_N:  D8 = D8(6); // 270 grad
pub const D8_NE: D8 = D8(7);
pub const D8_MIRROR_VERTICAL: D8 = D8(8);
pub const D8_9: D8 = D8(9);
pub const D8_10: D8 = D8(10);
pub const D8_11: D8 = D8(11);
pub const D8_MIRROR_HORIZONTAL: D8 = D8(12);
pub const D8_13: D8 = D8(13);
pub const D8_REVERSE_DIAGONAL: D8 = D8(14);
pub const D8_15: D8 = D8(15);

impl D8 {
    #[inline] pub fn ux(self) -> F { UX_F[self.0 as usize] }
    #[inline] pub fn uy(self) -> F { UY_F[self.0 as usize] }
    #[inline] pub fn vx(self) -> F { VX_F[self.0 as usize] }
    #[inline] pub fn vy(self) -> F { VY_F[self.0 as usize] }

    // Adds 180 degrees to rotation. Commutative operation.
    #[inline]
    pub fn rotate180(self) -> Self { D8(self.0 ^ 4) }

    #[inline]
    pub fn inv(self) -> Self {
        if self.0 & 8 != 0 {
            D8(self.0 & 15)
        } else {
            D8((-self.0) & 7)
        }
    }

    // Direction of main vector can be horizontal, vertical or diagonal.
    // Some objects work with vertical directions different.
    #[inline]
    pub fn is_vertical(self) -> bool { self.0 & 3 == 2 }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        D8(ROTATOR[self.0 as usize][other.0 as usize])
    }
    #[inline]
    pub fn sub(self, other: Self) -> Self {
        D8(ROTATOR[self.0 as usize][other.inv().0 as usize])
    }

    #[inline]
    pub fn by_direction(dx: F, dy: F) -> Self {
        if dx.abs() * 2.0 <= dy.abs() {
            if dy >= 0.0 { D8_S } else { D8_N }
        } else if dy.abs() * 2.0 <= dx.abs() {
            if dx > 0.0 { D8_E } else { D8_W }
        } else if dy > 0.0 {
            if dx > 0.0 { D8_SE } else { D8_SW }
        } else if dx > 0.0 { D8_NE } else { D8_NW }
    }
}

#[test]
fn all() {
    assert_eq!(D8_E.rotate180(), D8_W);
    assert_eq!(D8_SE.rotate180(), D8_NW);
    assert_eq!(D8_S.rotate180(), D8_N);
    assert_eq!(D8_SW.rotate180(), D8_NE);
    assert_eq!(D8_W.rotate180(), D8_E);
    assert_eq!(D8_NW.rotate180(), D8_SE);
    assert_eq!(D8_N.rotate180(), D8_S);
    assert_eq!(D8_NE.rotate180(), D8_SW);

    assert_eq!(D8_S.add(D8_SE).sub(D8_SE), D8_S, "smoke");

    let mut mul = Vec::new();
    for i in 0..16 {
        let mut row = Vec::new();
        for j in 0..16 {
            let ux = (UX[i] * UX[j] + VX[i] * UY[j]).signum();
            let uy = (UY[i] * UX[j] + VY[i] * UY[j]).signum();
            let vx = (UX[i] * VX[j] + VX[i] * VY[j]).signum();
            let vy = (UY[i] * VX[j] + VY[i] * VY[j]).signum();

            for k in 0..16 {
                if UX[k] == ux && UY[k] == uy && VX[k] == vx && VY[k] == vy {
                    row.push(k as I);
                    break;
                }
            }
        }
        mul.push(row);
    }

    for i in 0..16 {
        assert_eq!(&mul[i], &ROTATOR[i], "gen");
    }

/*
    (0..16).map(|i| {
        (UX[i], UY[i], VX[i], VY[i], 0, 0)
    }).collect()
*/
}

    /*
    /**
    * Helps sprite to compensate texture packer rotation.
    *
    * @memberof PIXI.GroupD8
    * @param {PIXI.Matrix} matrix - sprite world matrix
    * @param {number} rotation - The rotation factor to use.
    * @param {number} tx - sprite anchoring
    * @param {number} ty - sprite anchoring
    */
    matrixAppendRotationInv: (matrix, rotation, tx = 0, ty = 0) =>
    {
        // Packer used "rotation", we use "inv(rotation)"
        const mat = tempMatrices[GroupD8.inv(rotation)];

        mat.tx = tx;
        mat.ty = ty;
matrix.append(mat);
*/
