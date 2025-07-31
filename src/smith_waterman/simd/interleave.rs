use std::simd::{num::SimdUint, LaneCount, Simd, SupportedLaneCount};

#[inline]
pub fn interleave_simd<const W: usize, const L: usize>(strs: [&str; L]) -> [Simd<u16, L>; W]
where
    LaneCount<L>: SupportedLaneCount,
{
    // Ensure the strings are all the length of W
    let strs = std::array::from_fn(|i| {
        let mut tmp = [0u8; W];
        tmp[0..strs[i].len()].copy_from_slice(strs[i].as_bytes());
        tmp
    });

    let chunk_count = W.div_ceil(L);
    let mut interleaved = [Simd::splat(0); W];

    for chunk_idx in 0..chunk_count {
        let offset = chunk_idx * L;

        if L == 1 {
            let simds = to_simd::<W, 1, L>(strs, offset);
            let simds: [Simd<u16, L>; 1] = std::array::from_fn(|i| simds[i].cast::<u16>());
            if offset + L > W {
                interleaved[offset..W].copy_from_slice(&simds[0..(W - offset)]);
            } else {
                interleaved[offset..(offset + L)].copy_from_slice(&simds);
            }
        } else if L == 2 {
            let simds = interleave_2(to_simd::<W, 2, L>(strs, offset));
            if offset + L > W {
                interleaved[offset..W].copy_from_slice(&simds[0..(W - offset)]);
            } else {
                interleaved[offset..(offset + L)].copy_from_slice(&simds);
            }
        } else if L == 4 {
            let simds = interleave_4(to_simd::<W, 4, L>(strs, offset));
            if offset + L > W {
                interleaved[offset..W].copy_from_slice(&simds[0..(W - offset)]);
            } else {
                interleaved[offset..(offset + L)].copy_from_slice(&simds);
            }
        } else if L == 8 {
            let simds = interleave_8(to_simd::<W, 8, L>(strs, offset));
            if offset + L > W {
                interleaved[offset..W].copy_from_slice(&simds[0..(W - offset)]);
            } else {
                interleaved[offset..(offset + L)].copy_from_slice(&simds);
            }
        } else if L == 16 {
            let simds = interleave_16(to_simd::<W, 16, L>(strs, offset));
            if offset + L > W {
                interleaved[offset..W].copy_from_slice(&simds[0..(W - offset)]);
            } else {
                interleaved[offset..(offset + L)].copy_from_slice(&simds);
            }
        } else if L == 32 {
            let simds = interleave_32(to_simd::<W, 32, L>(strs, offset));
            if offset + L > W {
                interleaved[offset..W].copy_from_slice(&simds[0..(W - offset)]);
            } else {
                interleaved[offset..(offset + L)].copy_from_slice(&simds);
            }
        } else {
            panic!("Lanes (L) must be 1, 2, 4, 8, 16 or 32");
        }
    }

    interleaved
}

fn to_simd<const IW: usize, const W: usize, const L: usize>(
    strs: [[u8; IW]; L],
    offset: usize,
) -> [Simd<u8, L>; W]
where
    LaneCount<L>: SupportedLaneCount,
{
    std::array::from_fn(|i| Simd::load_or_default(&strs[i][offset..(offset + L).min(IW)]))
}

fn interleave_2<const L: usize>(simds: [Simd<u8, L>; 2]) -> [Simd<u16, L>; 2]
where
    LaneCount<L>: SupportedLaneCount,
{
    let (a, b) = simds[0].interleave(simds[1]);

    [a.cast::<u16>(), b.cast::<u16>()]
}

fn interleave_4<const L: usize>(simds: [Simd<u8, L>; 4]) -> [Simd<u16, L>; 4]
where
    LaneCount<L>: SupportedLaneCount,
{
    let [a, b, c, d] = simds;

    // Distance 2
    let (a, c) = a.interleave(c);
    let (b, d) = b.interleave(d);

    // Distance 1
    let (a, b) = a.interleave(b);
    let (c, d) = c.interleave(d);

    [
        a.cast::<u16>(),
        b.cast::<u16>(),
        c.cast::<u16>(),
        d.cast::<u16>(),
    ]
}

fn interleave_8<const L: usize>(simds: [Simd<u8, L>; 8]) -> [Simd<u16, L>; 8]
where
    LaneCount<L>: SupportedLaneCount,
{
    let [a, b, c, d, e, f, g, h] = simds;

    // Distance 4
    let (a, e) = a.interleave(e);
    let (b, f) = b.interleave(f);
    let (c, g) = c.interleave(g);
    let (d, h) = d.interleave(h);

    // Distance 2
    let (a, c) = a.interleave(c);
    let (e, g) = e.interleave(g);

    let (b, d) = b.interleave(d);
    let (f, h) = f.interleave(h);

    // Distance 1
    let (a, b) = a.interleave(b);
    let (c, d) = c.interleave(d);
    let (e, f) = e.interleave(f);
    let (g, h) = g.interleave(h);

    [
        a.cast::<u16>(),
        b.cast::<u16>(),
        c.cast::<u16>(),
        d.cast::<u16>(),
        e.cast::<u16>(),
        f.cast::<u16>(),
        g.cast::<u16>(),
        h.cast::<u16>(),
    ]
}

fn interleave_16<const L: usize>(simds: [Simd<u8, L>; 16]) -> [Simd<u16, L>; 16]
where
    LaneCount<L>: SupportedLaneCount,
{
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = simds;

    // Distance 8
    let (a, i) = a.interleave(i);
    let (b, j) = b.interleave(j);
    let (c, k) = c.interleave(k);
    let (d, l) = d.interleave(l);
    let (e, m) = e.interleave(m);
    let (f, n) = f.interleave(n);
    let (g, o) = g.interleave(o);
    let (h, p) = h.interleave(p);

    // Distance 4
    let (a, e) = a.interleave(e);
    let (b, f) = b.interleave(f);
    let (c, g) = c.interleave(g);
    let (d, h) = d.interleave(h);

    let (i, m) = i.interleave(m);
    let (j, n) = j.interleave(n);
    let (k, o) = k.interleave(o);
    let (l, p) = l.interleave(p);

    // Distance 2
    let (a, c) = a.interleave(c);
    let (b, d) = b.interleave(d);

    let (e, g) = e.interleave(g);
    let (f, h) = f.interleave(h);

    let (i, k) = i.interleave(k);
    let (j, l) = j.interleave(l);

    let (m, o) = m.interleave(o);
    let (n, p) = n.interleave(p);

    // Distance 1
    let (a, b) = a.interleave(b);
    let (c, d) = c.interleave(d);
    let (e, f) = e.interleave(f);
    let (g, h) = g.interleave(h);
    let (i, j) = i.interleave(j);
    let (k, l) = k.interleave(l);
    let (m, n) = m.interleave(n);
    let (o, p) = o.interleave(p);

    [
        a.cast::<u16>(),
        b.cast::<u16>(),
        c.cast::<u16>(),
        d.cast::<u16>(),
        e.cast::<u16>(),
        f.cast::<u16>(),
        g.cast::<u16>(),
        h.cast::<u16>(),
        i.cast::<u16>(),
        j.cast::<u16>(),
        k.cast::<u16>(),
        l.cast::<u16>(),
        m.cast::<u16>(),
        n.cast::<u16>(),
        o.cast::<u16>(),
        p.cast::<u16>(),
    ]
}

fn interleave_32<const L: usize>(simds: [Simd<u8, L>; 32]) -> [Simd<u16, L>; 32]
where
    LaneCount<L>: SupportedLaneCount,
{
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, a2, b2, c2, d2, e2, f2] =
        simds;

    // Distance 16
    let (a, q) = a.interleave(q);
    let (b, r) = b.interleave(r);
    let (c, s) = c.interleave(s);
    let (d, t) = d.interleave(t);
    let (e, u) = e.interleave(u);
    let (f, v) = f.interleave(v);
    let (g, w) = g.interleave(w);
    let (h, x) = h.interleave(x);
    let (i, y) = i.interleave(y);
    let (j, z) = j.interleave(z);
    let (k, a2) = k.interleave(a2);
    let (l, b2) = l.interleave(b2);
    let (m, c2) = m.interleave(c2);
    let (n, d2) = n.interleave(d2);
    let (o, e2) = o.interleave(e2);
    let (p, f2) = p.interleave(f2);

    // Distance 8
    let (a, i) = a.interleave(i);
    let (b, j) = b.interleave(j);
    let (c, k) = c.interleave(k);
    let (d, l) = d.interleave(l);
    let (e, m) = e.interleave(m);
    let (f, n) = f.interleave(n);
    let (g, o) = g.interleave(o);
    let (h, p) = h.interleave(p);

    let (q, y) = q.interleave(y);
    let (r, z) = r.interleave(z);
    let (s, a2) = s.interleave(a2);
    let (t, b2) = t.interleave(b2);
    let (u, c2) = u.interleave(c2);
    let (v, d2) = v.interleave(d2);
    let (w, e2) = w.interleave(e2);
    let (x, f2) = x.interleave(f2);

    // Distance 4
    let (a, e) = a.interleave(e);
    let (b, f) = b.interleave(f);
    let (c, g) = c.interleave(g);
    let (d, h) = d.interleave(h);

    let (i, m) = i.interleave(m);
    let (j, n) = j.interleave(n);
    let (k, o) = k.interleave(o);
    let (l, p) = l.interleave(p);

    let (q, u) = q.interleave(u);
    let (r, v) = r.interleave(v);
    let (s, w) = s.interleave(w);
    let (t, x) = t.interleave(x);

    let (y, c2) = y.interleave(c2);
    let (z, d2) = z.interleave(d2);
    let (a2, e2) = a2.interleave(e2);
    let (b2, f2) = b2.interleave(f2);

    // Distance 2
    let (a, c) = a.interleave(c);
    let (b, d) = b.interleave(d);

    let (e, g) = e.interleave(g);
    let (f, h) = f.interleave(h);

    let (i, k) = i.interleave(k);
    let (j, l) = j.interleave(l);

    let (m, o) = m.interleave(o);
    let (n, p) = n.interleave(p);

    let (q, s) = q.interleave(s);
    let (r, t) = r.interleave(t);

    let (u, w) = u.interleave(w);
    let (v, x) = v.interleave(x);

    let (y, a2) = y.interleave(a2);
    let (z, b2) = z.interleave(b2);

    let (c2, e2) = c2.interleave(e2);
    let (d2, f2) = d2.interleave(f2);

    [
        a.cast::<u16>(),
        b.cast::<u16>(),
        c.cast::<u16>(),
        d.cast::<u16>(),
        e.cast::<u16>(),
        f.cast::<u16>(),
        g.cast::<u16>(),
        h.cast::<u16>(),
        i.cast::<u16>(),
        j.cast::<u16>(),
        k.cast::<u16>(),
        l.cast::<u16>(),
        m.cast::<u16>(),
        n.cast::<u16>(),
        o.cast::<u16>(),
        p.cast::<u16>(),
        q.cast::<u16>(),
        r.cast::<u16>(),
        s.cast::<u16>(),
        t.cast::<u16>(),
        u.cast::<u16>(),
        v.cast::<u16>(),
        w.cast::<u16>(),
        x.cast::<u16>(),
        y.cast::<u16>(),
        z.cast::<u16>(),
        a2.cast::<u16>(),
        b2.cast::<u16>(),
        c2.cast::<u16>(),
        d2.cast::<u16>(),
        e2.cast::<u16>(),
        f2.cast::<u16>(),
    ]
}

#[cfg(test)]
mod tests {
    use std::simd::{LaneCount, Simd, SupportedLaneCount};

    use super::interleave_simd;

    fn interleave_ref<const W: usize, const L: usize>(strs: [&str; L]) -> [Simd<u16, L>; W]
    where
        LaneCount<L>: SupportedLaneCount,
    {
        std::array::from_fn(|i| {
            Simd::from_array(std::array::from_fn(|j| strs[j].as_bytes()[i] as u16))
        })
    }

    fn assert_matrix_eq<const L: usize, const W: usize>(a: [Simd<u16, L>; W], b: [[u8; L]; W])
    where
        LaneCount<L>: SupportedLaneCount,
    {
        let a = a.map(|a| {
            a.to_array()
                .into_iter()
                .map(|x| x as u8)
                .collect::<Vec<_>>()
        });
        assert_eq!(a, b);
    }

    #[test]
    fn test_interleave_simd_2() {
        let strs = ["ab", "cd"];
        let interleaved = interleave_simd::<2, 2>(strs);
        assert_matrix_eq(interleaved, [[b'a', b'c'], [b'b', b'd']]);
    }

    #[test]
    fn test_interleave_simd_chunks_2() {
        let strs = ["abcd", "efgh"];
        let interleaved = interleave_simd::<4, 2>(strs);
        assert_matrix_eq(
            interleaved,
            [[b'a', b'e'], [b'b', b'f'], [b'c', b'g'], [b'd', b'h']],
        );
    }

    #[test]
    fn test_interleave_simd_4() {
        let strs = ["abcd", "efgh", "ijkl", "mnop"];
        let interleaved = interleave_simd::<4, 4>(strs);
        assert_matrix_eq(
            interleaved,
            [
                [b'a', b'e', b'i', b'm'],
                [b'b', b'f', b'j', b'n'],
                [b'c', b'g', b'k', b'o'],
                [b'd', b'h', b'l', b'p'],
            ],
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_interleave_simd_8() {
        let strs = ["abcdefgh", "ijklmnop", "qrstuvwx", "yzABCDEF", "GHIJKLMN", "OPQRSTUV", "WXYZ1234", "56789012"];
        let interleaved = interleave_simd::<8, 8>(strs);

        assert_matrix_eq(
            interleaved,
            [
                [b'a', b'i', b'q', b'y', b'G', b'O', b'W', b'5'],
                [b'b', b'j', b'r', b'z', b'H', b'P', b'X', b'6'],
                [b'c', b'k', b's', b'A', b'I', b'Q', b'Y', b'7'],
                [b'd', b'l', b't', b'B', b'J', b'R', b'Z', b'8'],
                [b'e', b'm', b'u', b'C', b'K', b'S', b'1', b'9'],
                [b'f', b'n', b'v', b'D', b'L', b'T', b'2', b'0'],
                [b'g', b'o', b'w', b'E', b'M', b'U', b'3', b'1'],
                [b'h', b'p', b'x', b'F', b'N', b'V', b'4', b'2'],
            ],
        );
    }
}
