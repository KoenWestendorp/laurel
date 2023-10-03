use arraystring::{typenum::U5, ArrayString};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Atom {
    // 5 positions, integer.
    /// Residue number.
    pub(crate) resnum: u32,
    // 5 characters.
    /// Residue name.
    pub(crate) resname: ArrayString<U5>,
    // 5 characters.
    /// Atom name.
    pub(crate) atomname: ArrayString<U5>,
    // 5 positions, integer.
    /// Atom number.
    pub(crate) atomnum: u32,
    // In nm, x y z in 3 columns, each 8 positions with 3 decimal places.
    /// Position (nm).
    pub(crate) position: Vec3,
    // TODO: Consider adding velocity. Seems unnecessary for now, though.
    // /// Velocity (nm/ps or km/s).
    // velocity: Vec3, // In nm/ps, x y z in 3 columns, each 8 positions with 4 decimal places.
}

#[derive(Debug, Clone, PartialEq)]
pub struct Structure {
    pub(crate) title: String,
    pub(crate) atoms: Vec<Atom>,
    pub(crate) box_vecs: [f32; 9],
}

impl Structure {
    /// Return the center of the positions of all atoms in the structure.
    ///
    /// All atoms are weighed equally, so this is not necessarily the center of mass. In other
    /// words, the average position of all the atoms is returned.
    ///
    /// If there are not atoms in the system, a zero vector is returned.
    pub fn center(&self) -> Vec3 {
        self.atoms
            .iter()
            .fold(Vec3::ZERO, |acc, &Atom { position, .. }| acc + position)
            / self.n_atoms() as f32
    }

    /// Centers all atoms of this [`Structure`] such that the [`Structure::center`] is zero.
    pub fn center_structure(&mut self) {
        let center = self.center();
        self.atoms
            .iter_mut()
            .for_each(|atom| atom.position -= center);
    }

    /// Returns the number of atoms in this [`Structure`].
    pub fn n_atoms(&self) -> usize {
        self.atoms.len()
    }

    pub fn min_z(&self) -> f32 {
        let mut min_z = if let Some(first) = self.atoms.first() {
            first.position.z
        } else {return 0.0};

        for Atom { position, .. } in &self.atoms[1..] {
            if position.z < min_z { min_z = position.z }
        }

        min_z
    }

    pub fn max_z(&self) -> f32 {
        let mut max_z = if let Some(first) = self.atoms.first() {
            first.position.z
        } else {return 0.0};

        for Atom { position, .. } in &self.atoms[1..] {
            if position.z > max_z { max_z = position.z }
        }

        max_z
    }
}

impl Structure {
    // TODO: Actually check whether assuming ascii where possible is valid. Cannot imagine it is
    // not valid though.
    // TODO: In that same vain, perhaps actually implement an error system for this :/
    /// Read a string in Gromacs `.gro` format to a [`Structure`].
    pub fn from_gro(gro: String) -> Result<Self, String> {
        let mut lines = gro.lines();
        let title = String::from(lines.next().expect("too short file").trim());
        let n_atoms = lines
            .next()
            .expect("too short file")
            .trim()
            .parse()
            .expect("invalid n_atoms integer at line 2 of the gro file");

        let mut atoms = Vec::with_capacity(n_atoms);
        for _ in 0..n_atoms {
            let line = lines
                .next()
                .expect("end of file before all atoms have been specified");
            let atom = Atom {
                resnum: line[0..5].trim().parse().expect("bad resnum integer"),
                resname: line[5..10].trim().try_into().unwrap(), // We know that the length <= 5.
                atomname: line[10..15].trim().try_into().unwrap(),
                atomnum: line[15..20].trim().parse().expect("bad atomnum integer"),
                position: {
                    // TODO: Is this nicer?? vvv
                    // [0, 1, 2]
                    //     .map(|i| i * 8)
                    //     .map(|i| line[i..i + 8].parse::<f32>().expect("bad position float"))
                    //     .into()
                    [&line[20..28], &line[28..36], &line[36..44]]
                        .map(|v| v.trim().parse::<f32>().expect("bad position float"))
                        .into()
                },
            };
            atoms.push(atom);
        }

        let mut box_vecs: [f32; 9] = Default::default();
        let mut box_line = lines
            .next()
            .expect("too short file")
            .split_ascii_whitespace()
            .map(|v| v.parse::<f32>().ok());
        let [v1x, v2y, v3z] = box_line
            .next_chunk()
            .expect("bad box vectors")
            .map(|v| v.expect("bad first box vector triplet"));
        box_vecs[0..3].copy_from_slice(&[v1x, v2y, v3z]);
        if let [Some(v1y), Some(v1z), Some(v2x), Some(v2z), Some(v3x), Some(v3y)] =
            box_line.collect::<Vec<_>>()[..]
        {
            box_vecs[3..].copy_from_slice(&[v1y, v1z, v2x, v2z, v3x, v3y]);
        }

        Ok(Structure {
            title,
            atoms,
            box_vecs,
        })
    }
}
