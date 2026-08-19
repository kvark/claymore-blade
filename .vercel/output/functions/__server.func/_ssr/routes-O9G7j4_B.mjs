import { o as __toESM } from "../_runtime.mjs";
import { R as require_react, y as require_jsx_runtime } from "../_libs/@tanstack/react-router+[...].mjs";
import { r as signOut, t as authClient } from "./client-CLfOuzi9.mjs";
import { t as create } from "../_libs/zustand.mjs";
//#region node_modules/.nitro/vite/services/ssr/assets/routes-O9G7j4_B.js
var import_react = /* @__PURE__ */ __toESM(require_react());
var import_jsx_runtime = require_jsx_runtime();
/**
* Current user + loading state. Same behavior in live preview and when deployed:
*   - Auth enabled (default) -> the real signed-in user; `user` is `null` while
*                            the session resolves (`isPending: true`) and when
*                            signed out (`isPending: false`). Session comes from
*                            Better Auth `useSession()` → `/api/auth/get-session`
*                            (cookie when deployed; bearer in live preview).
*   - Auth disabled (`VITE_AUTH_ENABLED=false`) -> `DEV_USER`, never pending.
*
* Protect a route by waiting out `isPending` before acting on `user` —
* redirecting on `user: null` alone bounces signed-in visitors to sign-in on
* every hard reload:
*
*   import { RedirectToSignIn } from "@/lib/auth/gates";
*   const { user, isPending } = useCurrentUserState();
*   if (isPending) return null;              // still resolving — don't redirect yet
*   if (!user) return <RedirectToSignIn />;  // definitely signed out
*
* `authEnabled` is a module-level constant fixed at load, so the guarded hook
* call keeps a stable hook order across every render of a given component.
*/
function useCurrentUserState() {
	const { data, isPending } = authClient.useSession();
	const user = data?.user;
	return {
		user: user ? {
			id: user.id,
			displayName: user.name ?? null,
			primaryEmail: user.email ?? null,
			profileImageUrl: user.image ?? null,
			isDevFallback: false
		} : null,
		isPending
	};
}
/**
* Convenience view of `useCurrentUserState().user` for display (e.g.
* `user?.displayName ?? "Guest"`). NOTE: `null` means *loading OR signed out* —
* for redirects/guards use `useCurrentUserState()` and check `isPending`.
*/
function useCurrentUser() {
	return useCurrentUserState().user;
}
/** Render children only when a user is present (real session, or the disabled-auth dev user). */
function SignedIn({ children }) {
	const { user } = useCurrentUserState();
	return user ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_jsx_runtime.Fragment, { children }) : null;
}
/**
* Render children only once we KNOW the visitor is signed out (`isPending` has
* cleared and there is no user). Hidden while the session is still loading.
*/
function SignedOut({ children }) {
	const { user, isPending } = useCurrentUserState();
	if (isPending || user) return null;
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_jsx_runtime.Fragment, { children });
}
/**
* Minimal signed-in identity chip + sign-out. Restyle freely (see the
* `design-ui` skill). Sign-out is only shown when auth is enabled (the
* disabled-auth dev user has nothing to sign out of).
*/
function UserButton() {
	const user = useCurrentUser();
	if (!user) return null;
	const label = user.displayName ?? user.primaryEmail ?? "Account";
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "flex items-center gap-2",
		children: [
			user.profileImageUrl ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
				src: user.profileImageUrl,
				alt: "",
				className: "h-8 w-8 rounded-full object-cover"
			}) : /* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
				className: "grid h-8 w-8 place-items-center rounded-full bg-black/10 text-sm font-medium dark:bg-white/20",
				children: label.charAt(0).toUpperCase()
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
				className: "text-sm font-medium",
				children: label
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
				type: "button",
				onClick: () => void signOut(),
				className: "cursor-pointer text-sm underline-offset-4 opacity-70 hover:underline",
				children: "Sign out"
			})
		]
	});
}
var SKILLS = {
	cut: {
		id: "cut",
		name: "Cut",
		blurb: "A clean greatsword line. The first thing they teach you.",
		ap: 1,
		trans: 0,
		yoki: 0,
		shape: "single",
		range: 1,
		pa: "S",
		pd: "A",
		power: 18
	},
	aimed: {
		id: "aimed",
		name: "Aimed Cut",
		blurb: "Trade certainty for a limb. Core if you can see it.",
		ap: 1,
		trans: 20,
		yoki: 4,
		shape: "single",
		range: 1,
		pa: "C",
		pd: "A",
		power: 16,
		aimed: true
	},
	guard: {
		id: "guard",
		name: "Guard",
		blurb: "Blade up. The next blow wants to be a block.",
		ap: 1,
		trans: 0,
		yoki: 0,
		shape: "self",
		range: 0,
		pa: "C",
		pd: "S",
		power: 0,
		self: true,
		guard: 3
	},
	vault: {
		id: "vault",
		name: "Vault",
		blurb: "Leave the hex. Arrive two away.",
		ap: 1,
		trans: 20,
		yoki: 3,
		shape: "leap",
		range: 2,
		pa: "A",
		pd: "A",
		power: 0,
		self: true,
		move: 2
	},
	knit: {
		id: "knit",
		name: "Knit",
		blurb: "Force meat back together. Hurts less than it should.",
		ap: 1,
		trans: 20,
		yoki: 6,
		shape: "self",
		range: 0,
		pa: "W",
		pd: "S",
		power: 0,
		self: true,
		heal: 22
	},
	rest: {
		id: "rest",
		name: "Rest",
		blurb: "Lower the bar. Breathe. The thing inside hates this.",
		ap: 2,
		trans: 0,
		yoki: 0,
		shape: "self",
		range: 0,
		pa: "C",
		pd: "C",
		power: 0,
		self: true,
		heal: 8,
		transDelta: -12
	},
	drop: {
		id: "drop",
		name: "Trans Drop",
		blurb: "A human hand on your wrist. The bar falls.",
		ap: 1,
		trans: 0,
		yoki: 0,
		shape: "self",
		range: 0,
		pa: "C",
		pd: "C",
		power: 0,
		self: true,
		transDelta: -28
	},
	read: {
		id: "read",
		name: "Read Energy",
		blurb: "See the next swing before the muscle knows.",
		ap: 1,
		trans: 20,
		yoki: 5,
		shape: "self",
		range: 0,
		pa: "P",
		pd: "C",
		power: 0,
		self: true,
		telegraph: true
	},
	flash: {
		id: "flash",
		name: "Flash Blade",
		blurb: "All yoki into the sword arm. Too fast to see. Guard in the same breath.",
		ap: 1,
		trans: 40,
		yoki: 10,
		shape: "sweep",
		range: 1,
		pa: "A",
		pd: "A",
		power: 14,
		strikes: true,
		guard: 2
	},
	wind: {
		id: "wind",
		name: "Wind Shear",
		blurb: "Draw, cut, sheath. Slower than Flash. It does not miss.",
		ap: 1,
		trans: 40,
		yoki: 8,
		shape: "line",
		range: 3,
		length: 3,
		pa: "C",
		pd: "A",
		power: 20
	},
	stretch: {
		id: "stretch",
		name: "Stretch Arm",
		blurb: "The arm leaves the shoulder and arrives four hexes later.",
		ap: 1,
		trans: 40,
		yoki: 8,
		shape: "line",
		range: 4,
		length: 4,
		pa: "S",
		pd: "A",
		power: 17
	},
	phantom: {
		id: "phantom",
		name: "Phantom Step",
		blurb: "You are already gone. A lie stays behind.",
		ap: 1,
		trans: 40,
		yoki: 8,
		shape: "leap",
		range: 3,
		pa: "A",
		pd: "P",
		power: 0,
		self: true,
		move: 3,
		afterimage: true
	},
	spiral: {
		id: "spiral",
		name: "Spiral Drill",
		blurb: "Twenty-one turns in the shoulder. Everything in that hex ends.",
		ap: 2,
		trans: 60,
		yoki: 14,
		shape: "single",
		range: 1,
		pa: "S",
		pd: "S",
		power: 38,
		unblockable: true
	},
	regen: {
		id: "regen",
		name: "Unmake Wound",
		blurb: "A lost arm is an argument. She wins it.",
		ap: 1,
		trans: 40,
		yoki: 12,
		shape: "self",
		range: 0,
		pa: "W",
		pd: "S",
		power: 0,
		self: true,
		heal: 40
	},
	hide: {
		id: "hide",
		name: "Hide Energy",
		blurb: "Fold the light inward. Distant eyes skip you.",
		ap: 1,
		trans: 0,
		yoki: 3,
		shape: "self",
		range: 0,
		pa: "C",
		pd: "P",
		power: 0,
		self: true
	},
	claw: {
		id: "claw",
		name: "Rend",
		blurb: "A yoma's only thesis.",
		ap: 1,
		trans: 0,
		yoki: 0,
		shape: "single",
		range: 1,
		pa: "S",
		pd: "A",
		power: 16
	},
	lunge: {
		id: "lunge",
		name: "Lunge",
		blurb: "Too many joints. Too much reach.",
		ap: 1,
		trans: 20,
		yoki: 0,
		shape: "line",
		range: 2,
		length: 2,
		pa: "A",
		pd: "A",
		power: 14
	},
	ripple: {
		id: "ripple",
		name: "Ripple Edge",
		blurb: "A ring of cutting air. It grows. Do not stand in the next one.",
		ap: 2,
		trans: 40,
		yoki: 8,
		shape: "ripple",
		range: 0,
		pa: "S",
		pd: "A",
		power: 18
	},
	slam: {
		id: "slam",
		name: "Core Slam",
		blurb: "The body falls on three hexes.",
		ap: 1,
		trans: 20,
		yoki: 0,
		shape: "blast",
		range: 1,
		pa: "S",
		pd: "S",
		power: 20
	},
	whip: {
		id: "whip",
		name: "Tail Whip",
		blurb: "A line through the board. The tail is the weapon.",
		ap: 1,
		trans: 0,
		yoki: 0,
		shape: "line",
		range: 3,
		length: 3,
		pa: "S",
		pd: "A",
		power: 15
	}
};
var WARRIORS = {
	kira: {
		id: "kira",
		name: "Kira",
		title: "No. 47",
		rank: 47,
		side: "player",
		portrait: "/art/kira.jpg",
		sprite: "/sprites/kira.png",
		stats: {
			S: 6,
			A: 8,
			C: 9,
			P: 11,
			W: 7
		},
		skills: [
			"cut",
			"aimed",
			"guard",
			"read",
			"flash",
			"wind",
			"knit",
			"rest",
			"drop"
		],
		trans: 8,
		footprint: [{
			q: 0,
			r: 0
		}],
		coreIndex: 0,
		parts: [{
			id: "body",
			name: "Body",
			hexIndex: 0,
			hp: 48
		}],
		color: "#c8ccd4"
	},
	vespera: {
		id: "vespera",
		name: "Vespera",
		title: "No. 6 · Phantom",
		rank: 6,
		side: "player",
		portrait: "/art/vespera.jpg",
		sprite: "/sprites/kira.png",
		stats: {
			S: 8,
			A: 12,
			C: 10,
			P: 9,
			W: 8
		},
		skills: [
			"cut",
			"guard",
			"phantom",
			"aimed",
			"rest"
		],
		trans: 18,
		footprint: [{
			q: 0,
			r: 0
		}],
		coreIndex: 0,
		parts: [{
			id: "body",
			name: "Body",
			hexIndex: 0,
			hp: 62
		}],
		color: "#9aa3ad"
	},
	rhea: {
		id: "rhea",
		name: "Rhea",
		title: "No. 22 · Long Arm",
		rank: 22,
		side: "player",
		portrait: "/art/rhea.jpg",
		sprite: "/sprites/kira.png",
		stats: {
			S: 10,
			A: 8,
			C: 7,
			P: 7,
			W: 6
		},
		skills: [
			"cut",
			"stretch",
			"guard",
			"rest"
		],
		trans: 22,
		footprint: [{
			q: 0,
			r: 0
		}],
		coreIndex: 0,
		parts: [{
			id: "body",
			name: "Body",
			hexIndex: 0,
			hp: 70
		}],
		color: "#b7a48a"
	},
	nessa: {
		id: "nessa",
		name: "Nessa",
		title: "No. 15 · Unbroken",
		rank: 15,
		side: "player",
		portrait: "/art/nessa.jpg",
		sprite: "/sprites/kira.png",
		stats: {
			S: 11,
			A: 6,
			C: 11,
			P: 6,
			W: 8
		},
		skills: [
			"cut",
			"guard",
			"regen",
			"knit",
			"rest"
		],
		trans: 16,
		footprint: [{
			q: 0,
			r: 0
		}],
		coreIndex: 0,
		parts: [{
			id: "body",
			name: "Body",
			hexIndex: 0,
			hp: 88
		}],
		color: "#8d9490"
	}
};
var ENEMIES = {
	yoma: {
		id: "yoma",
		name: "Yoma",
		title: "Flesh-eater",
		side: "enemy",
		portrait: "/sprites/yoma.png",
		sprite: "/sprites/yoma.png",
		stats: {
			S: 7,
			A: 6,
			C: 3,
			P: 5,
			W: 2
		},
		skills: ["claw", "lunge"],
		trans: 35,
		footprint: [{
			q: 0,
			r: 0
		}],
		coreIndex: 0,
		parts: [{
			id: "body",
			name: "Body",
			hexIndex: 0,
			hp: 36
		}],
		color: "#8a3a32"
	},
	yoma_stretch: {
		id: "yoma_stretch",
		name: "Long Yoma",
		title: "Jointed",
		side: "enemy",
		portrait: "/sprites/yoma.png",
		sprite: "/sprites/yoma.png",
		stats: {
			S: 8,
			A: 7,
			C: 4,
			P: 6,
			W: 3
		},
		skills: ["claw", "stretch"],
		trans: 48,
		footprint: [{
			q: 0,
			r: 0
		}],
		coreIndex: 0,
		parts: [{
			id: "body",
			name: "Body",
			hexIndex: 0,
			hp: 44
		}],
		color: "#7a2e28"
	},
	ophel: {
		id: "ophel",
		name: "Ophel",
		title: "The Ripple · Awakened No. 4",
		rank: 4,
		side: "enemy",
		portrait: "/art/ophel.jpg",
		sprite: "/sprites/ophel.png",
		stats: {
			S: 13,
			A: 10,
			C: 8,
			P: 8,
			W: 6
		},
		skills: [
			"ripple",
			"claw",
			"slam"
		],
		trans: 88,
		footprint: [
			{
				q: 0,
				r: 0
			},
			{
				q: 1,
				r: 0
			},
			{
				q: 2,
				r: 0
			},
			{
				q: 0,
				r: -1
			}
		],
		coreIndex: 0,
		parts: [
			{
				id: "core",
				name: "Core",
				hexIndex: 0,
				hp: 70
			},
			{
				id: "torso",
				name: "Torso",
				hexIndex: 1,
				hp: 40
			},
			{
				id: "tail",
				name: "Tail",
				hexIndex: 2,
				hp: 32,
				zone: "ripple"
			},
			{
				id: "arm",
				name: "Cutting Arm",
				hexIndex: 3,
				hp: 28,
				zone: "claw"
			}
		],
		color: "#d8d2c4"
	},
	worm: {
		id: "worm",
		name: "North Worm",
		title: "Awakened · 5 cells",
		side: "enemy",
		portrait: "/sprites/ophel.png",
		sprite: "/sprites/ophel.png",
		stats: {
			S: 14,
			A: 6,
			C: 5,
			P: 5,
			W: 4
		},
		skills: [
			"whip",
			"slam",
			"claw"
		],
		trans: 80,
		footprint: [
			{
				q: 0,
				r: 0
			},
			{
				q: 1,
				r: 0
			},
			{
				q: 2,
				r: 0
			},
			{
				q: 3,
				r: 0
			},
			{
				q: 4,
				r: 0
			}
		],
		coreIndex: 0,
		parts: [
			{
				id: "core",
				name: "Head",
				hexIndex: 0,
				hp: 55
			},
			{
				id: "n1",
				name: "Neck",
				hexIndex: 1,
				hp: 30
			},
			{
				id: "n2",
				name: "Thorax",
				hexIndex: 2,
				hp: 30
			},
			{
				id: "n3",
				name: "Trunk",
				hexIndex: 3,
				hp: 26
			},
			{
				id: "tail",
				name: "Tail",
				hexIndex: 4,
				hp: 24,
				zone: "whip"
			}
		],
		color: "#cfc6b0"
	}
};
var LOCATIONS = [
	{
		id: "dovra",
		name: "Dovra",
		region: "Lautrin",
		blurb: "Mud, goats, a well. Something has been using the well.",
		x: .3,
		y: .56,
		kind: "village",
		encounter: "dovra-yoma",
		deadline: 36
	},
	{
		id: "stonecross",
		name: "Stonecross",
		region: "Lautrin",
		blurb: "A market that pretends the Office is a weather report.",
		x: .36,
		y: .64,
		kind: "village"
	},
	{
		id: "hanrel",
		name: "Hanrel",
		region: "Lautrin",
		blurb: "Statues of a smiling No. 1 and the girl she pulled from a wagon.",
		x: .34,
		y: .46,
		kind: "city"
	},
	{
		id: "whitepeak",
		name: "Whitepeak",
		region: "Albas",
		blurb: "A shrine in the wind. Sisters come here to stop being sisters.",
		x: .46,
		y: .3,
		kind: "shrine"
	},
	{
		id: "paburo",
		name: "Paburo",
		region: "Highlands",
		blurb: "Joints in the trees. The nest has learned to reach.",
		x: .5,
		y: .44,
		kind: "wild",
		encounter: "paburo-nest",
		deadline: 72
	},
	{
		id: "lacroix",
		name: "Lacroix",
		region: "Border",
		blurb: "Search parties. They write your number down.",
		x: .58,
		y: .48,
		kind: "village"
	},
	{
		id: "gonal",
		name: "Gonal",
		region: "Mucha",
		blurb: "A road town that asked for No. 4. She answered as something else.",
		x: .48,
		y: .7,
		kind: "village",
		encounter: "gonal-ripple",
		deadline: 96
	},
	{
		id: "pietra",
		name: "Pietra",
		region: "Albas",
		blurb: "The north is gathering. Highest rank leads. The worm is already there.",
		x: .44,
		y: .18,
		kind: "city",
		encounter: "pietra-worm",
		deadline: 140
	},
	{
		id: "maw",
		name: "Witch's Maw",
		region: "Lautrin",
		blurb: "A keep that forgot its owners. Something older than a rank lives in the well.",
		x: .2,
		y: .38,
		kind: "keep"
	},
	{
		id: "staffold",
		name: "Staffold",
		region: "East",
		blurb: "The Office. Rank, orders, and the room where they brand you.",
		x: .74,
		y: .5,
		kind: "office"
	}
];
var ENCOUNTERS = {
	"dovra-yoma": {
		id: "dovra-yoma",
		title: "Dovra · the well",
		briefing: "Two of them wore neighbors yesterday. Cut is enough. Raise the bar if you must. Do not max it.",
		cols: 11,
		rows: 8,
		enemies: [{
			template: "yoma",
			origin: {
				q: 6,
				r: 2
			}
		}, {
			template: "yoma",
			origin: {
				q: 8,
				r: 5
			}
		}],
		playerOrigins: [
			{
				q: 2,
				r: 4
			},
			{
				q: 1,
				r: 3
			},
			{
				q: 1,
				r: 5
			}
		],
		reward: {
			karma: 8,
			raku: true,
			flag: "dovra-cleared"
		}
	},
	"paburo-nest": {
		id: "paburo-nest",
		title: "Paburo · the nest",
		briefing: "Vespera is already here. Rhea laughs when the first arm leaves a shoulder. Stay out of the long lines.",
		cols: 11,
		rows: 9,
		enemies: [
			{
				template: "yoma",
				origin: {
					q: 7,
					r: 2
				}
			},
			{
				template: "yoma",
				origin: {
					q: 9,
					r: 6
				}
			},
			{
				template: "yoma_stretch",
				origin: {
					q: 8,
					r: 4
				}
			}
		],
		playerOrigins: [
			{
				q: 1,
				r: 4
			},
			{
				q: 2,
				r: 2
			},
			{
				q: 2,
				r: 6
			}
		],
		reward: {
			rank: -4,
			karma: 10,
			recruit: ["vespera", "rhea"],
			flag: "paburo-cleared"
		}
	},
	"gonal-ripple": {
		id: "gonal-ripple",
		title: "Gonal · the Ripple",
		briefing: "Ophel is four hexes of wrong anatomy. Cut the arm. Cut the tail. The rings will still grow. Do not stand in the next one.",
		cols: 12,
		rows: 9,
		enemies: [{
			template: "ophel",
			origin: {
				q: 7,
				r: 4
			},
			facing: 3
		}],
		playerOrigins: [
			{
				q: 2,
				r: 3
			},
			{
				q: 1,
				r: 5
			},
			{
				q: 3,
				r: 6
			}
		],
		reward: {
			rank: -8,
			karma: 16,
			flag: "gonal-cleared"
		}
	},
	"pietra-worm": {
		id: "pietra-worm",
		title: "Pietra · the north",
		briefing: "Nessa will not die. The worm occupies five cells. Sever the tail or live in the whip. Highest rank leads. That is not you.",
		cols: 13,
		rows: 10,
		enemies: [{
			template: "worm",
			origin: {
				q: 7,
				r: 3
			},
			facing: 3
		}, {
			template: "yoma",
			origin: {
				q: 10,
				r: 7
			}
		}],
		playerOrigins: [
			{
				q: 2,
				r: 4
			},
			{
				q: 1,
				r: 6
			},
			{
				q: 3,
				r: 7
			}
		],
		reward: {
			rank: -6,
			karma: 20,
			recruit: ["nessa"],
			flag: "pietra-cleared"
		}
	}
};
function derived(stats) {
	return {
		hit: stats.A + stats.C,
		dodge: stats.A + stats.P + stats.W,
		detect: stats.P,
		move: Math.max(2, Math.round(stats.A / 3)),
		transMax: stats.C + stats.W,
		health: 28 + stats.S * 6,
		yoki: 8 + stats.W * 4,
		damage: stats.S,
		strikes: Math.max(1, Math.round(stats.A / 4))
	};
}
function locById(id) {
	return LOCATIONS.find((l) => l.id === id);
}
var HEX_DIRS = [
	{
		q: 1,
		r: 0
	},
	{
		q: 1,
		r: -1
	},
	{
		q: 0,
		r: -1
	},
	{
		q: -1,
		r: 0
	},
	{
		q: -1,
		r: 1
	},
	{
		q: 0,
		r: 1
	}
];
function hexKey(h) {
	return `${h.q},${h.r}`;
}
function hexEq(a, b) {
	return a.q === b.q && a.r === b.r;
}
function hexAdd(a, b) {
	return {
		q: a.q + b.q,
		r: a.r + b.r
	};
}
function rotate60(h, times) {
	let q = h.q;
	let r = h.r;
	const t = (times % 6 + 6) % 6;
	for (let i = 0; i < t; i++) {
		const nq = -r;
		const nr = q + r;
		q = nq;
		r = nr;
	}
	return {
		q,
		r
	};
}
function cubeRound(qf, rf) {
	const sf = -qf - rf;
	let q = Math.round(qf);
	let r = Math.round(rf);
	let s = Math.round(sf);
	const qd = Math.abs(q - qf);
	const rd = Math.abs(r - rf);
	const sd = Math.abs(s - sf);
	if (qd > rd && qd > sd) q = -r - s;
	else if (rd > sd) r = -q - s;
	return {
		q,
		r
	};
}
function axialToPixel(h, size) {
	return {
		x: size * Math.sqrt(3) * (h.q + h.r / 2),
		y: size * (3 / 2) * h.r
	};
}
function pixelToAxial(x, y, size) {
	return cubeRound((Math.sqrt(3) / 3 * x - 1 / 3 * y) / size, 2 / 3 * y / size);
}
function hexDistance(a, b) {
	return (Math.abs(a.q - b.q) + Math.abs(a.q + a.r - b.q - b.r) + Math.abs(a.r - b.r)) / 2;
}
function hexNeighbors(h) {
	return HEX_DIRS.map((d) => hexAdd(h, d));
}
function hexDisc(center, radius) {
	const out = [];
	for (let q = -radius; q <= radius; q++) {
		const rMin = Math.max(-radius, -q - radius);
		const rMax = Math.min(radius, -q + radius);
		for (let r = rMin; r <= rMax; r++) out.push({
			q: center.q + q,
			r: center.r + r
		});
	}
	return out;
}
function hexRing(center, radius) {
	if (radius <= 0) return [{ ...center }];
	return hexDisc(center, radius).filter((h) => hexDistance(center, h) === radius);
}
function hexLine(a, b) {
	const n = hexDistance(a, b);
	if (n === 0) return [{ ...a }];
	const out = [];
	for (let i = 0; i <= n; i++) {
		const t = i / n;
		out.push(cubeRound(a.q + (b.q - a.q) * t, a.r + (b.r - a.r) * t));
	}
	return out;
}
function hexCone(origin, facing, range) {
	const fwd = HEX_DIRS[(facing % 6 + 6) % 6];
	const fq = fwd.q;
	const fr = fwd.r;
	const fs = -fq - fr;
	const out = [];
	for (const h of hexDisc(origin, range)) {
		const d = hexDistance(origin, h);
		if (d === 0) continue;
		const dq = h.q - origin.q;
		const dr = h.r - origin.r;
		const ds = -dq - dr;
		if (fq * dq + fr * dr + fs * ds >= d - .5) out.push(h);
	}
	return out;
}
function hexSweep(origin, facing) {
	const f = (facing % 6 + 6) % 6;
	return [
		HEX_DIRS[(f + 5) % 6],
		HEX_DIRS[f],
		HEX_DIRS[(f + 1) % 6]
	].map((d) => hexAdd(origin, d));
}
function facingToward(from, to) {
	const dq = to.q - from.q;
	const dr = to.r - from.r;
	if (dq === 0 && dr === 0) return 0;
	let best = 0;
	let bestDot = -Infinity;
	HEX_DIRS.forEach((d, i) => {
		const ds = -d.q - d.r;
		const hs = -dq - dr;
		const dot = d.q * dq + d.r * dr + ds * hs;
		if (dot > bestDot) {
			bestDot = dot;
			best = i;
		}
	});
	return best;
}
function placeFootprint(shape, origin, facing) {
	return shape.map((h) => hexAdd(origin, rotate60(h, facing)));
}
function hexCorners(cx, cy, size) {
	const out = [];
	for (let i = 0; i < 6; i++) {
		const angle = Math.PI / 180 * (60 * i - 30);
		out.push({
			x: cx + size * Math.cos(angle),
			y: cy + size * Math.sin(angle)
		});
	}
	return out;
}
var Rng = class {
	s;
	constructor(seed) {
		this.s = seed >>> 0 || 1;
	}
	next() {
		this.s = 1664525 * this.s + 1013904223 >>> 0;
		return this.s / 4294967296;
	}
	int(min, max) {
		return Math.floor(this.next() * (max - min + 1)) + min;
	}
	pick(arr) {
		return arr[Math.floor(this.next() * arr.length)];
	}
	chance(p) {
		return this.next() < p;
	}
};
function inBounds(h, cols, rows) {
	return h.q >= 0 && h.r >= 0 && h.q < cols && h.r < rows;
}
function liveCells(u) {
	return placeFootprint(u.footprint, u.origin, u.facing).filter((_, i) => {
		const part = u.parts.find((p) => p.hexIndex === i);
		return part ? part.hp > 0 : true;
	});
}
function coreHex(u) {
	return placeFootprint(u.footprint, u.origin, u.facing)[u.coreIndex] ?? u.origin;
}
function occupiedMap(state, ignoreId) {
	const m = /* @__PURE__ */ new Map();
	for (const u of state.units) {
		if (u.dead || u.id === ignoreId) continue;
		for (const c of liveCells(u)) m.set(hexKey(c), u.id);
	}
	return m;
}
function spawnFromTemplate(t, id, origin, facing) {
	const d = derived(t.stats);
	const parts = t.parts.map((p) => ({
		...p,
		maxHp: p.hp
	}));
	const hp = parts.reduce((s, p) => s + p.hp, 0);
	return {
		id,
		templateId: t.id,
		name: t.name,
		title: t.title,
		rank: t.rank,
		side: t.side,
		portrait: t.portrait,
		sprite: t.sprite,
		origin,
		facing,
		footprint: t.footprint.map((h) => ({ ...h })),
		coreIndex: t.coreIndex,
		parts,
		hp,
		maxHp: hp,
		yoki: d.yoki,
		maxYoki: d.yoki,
		trans: t.trans,
		ap: 2,
		maxAp: 2,
		stats: { ...t.stats },
		skills: [...t.skills],
		statuses: [],
		raisedTransThisTurn: false,
		color: t.color,
		dead: false
	};
}
function createBattle(encounterId, partyIds, seed = Date.now() % 1e6 + 1) {
	const enc = ENCOUNTERS[encounterId];
	if (!enc) throw new Error(`missing encounter ${encounterId}`);
	const units = [];
	partyIds.slice(0, 3).forEach((pid, i) => {
		const t = WARRIORS[pid];
		if (!t) return;
		const origin = enc.playerOrigins[i] ?? enc.playerOrigins[0];
		units.push(spawnFromTemplate(t, `p-${t.id}`, origin, 0));
	});
	enc.enemies.forEach((e, i) => {
		const t = ENEMIES[e.template];
		if (!t) return;
		units.push(spawnFromTemplate(t, `e-${t.id}-${i}`, e.origin, e.facing ?? 3));
	});
	const terrain = {};
	const rng = new Rng(seed);
	for (let q = 0; q < enc.cols; q++) for (let r = 0; r < enc.rows; r++) {
		const roll = rng.next();
		terrain[`${q},${r}`] = roll > .92 ? "ruin" : roll > .84 ? "mud" : roll < .04 ? "water" : "grass";
	}
	for (const u of units) for (const c of liveCells(u)) terrain[hexKey(c)] = "grass";
	const order = [...units].sort((a, b) => b.stats.A + rng.int(0, 9) - (a.stats.A + rng.int(0, 9))).map((u) => u.id);
	const state = {
		id: encounterId,
		title: enc.title,
		seed,
		turn: 0,
		round: 1,
		order,
		units,
		terrain,
		cols: enc.cols,
		rows: enc.rows,
		zones: [],
		log: [],
		briefing: enc.briefing
	};
	pushLog(state, "info", `${enc.title}. ${units.filter((u) => u.side === "enemy").length} on the board.`);
	beginTurn(state);
	return state;
}
function pushLog(state, kind, text) {
	state.log.unshift({
		t: state.round * 10 + state.turn,
		text,
		kind
	});
	if (state.log.length > 40) state.log.pop();
}
function currentUnit(state) {
	const id = state.order[state.turn];
	return state.units.find((u) => u.id === id && !u.dead);
}
function living(state, side) {
	return state.units.filter((u) => !u.dead && (side ? u.side === side : true));
}
function beginTurn(state) {
	const u = currentUnit(state);
	if (!u) {
		advanceTurn(state);
		return;
	}
	u.ap = u.maxAp;
	u.raisedTransThisTurn = false;
	u.yoki = Math.min(u.maxYoki, u.yoki + 2);
	u.statuses = u.statuses.map((s) => ({
		...s,
		turns: s.turns - 1
	})).filter((s) => s.turns > 0);
	if (u.trans >= 90 && u.side === "player") {
		if (new Rng(state.seed + state.round * 17 + u.trans).chance(.25 + (u.trans - 90) / 80)) {
			u.ap = 0;
			pushLog(state, "trans", `${u.name} loses the bar. The turn is gone.`);
		}
	}
	if (u.statuses.some((s) => s.telegraph)) for (const e of living(state, u.side === "player" ? "enemy" : "player")) e.nextHint = pickAiSkill(state, e)?.name ?? "Advance";
	tickRipples(state, u.id);
}
function tickRipples(state, actorId) {
	const keep = [];
	for (const z of state.zones) {
		applyZone(state, hexRing(z.center, z.radius).filter((h) => inBounds(h, state.cols, state.rows)), z.power, z.pa, "A", actorId, false, false);
		pushLog(state, "info", `Ripple expands to ${z.radius}.`);
		if (z.radius < z.maxRadius) keep.push({
			...z,
			radius: z.radius + 1
		});
	}
	state.zones = keep;
}
function advanceTurn(state) {
	if (state.over) return;
	const check = () => {
		if (living(state, "enemy").length === 0) state.over = "win";
		if (living(state, "player").length === 0) state.over = "lose";
	};
	check();
	if (state.over) return;
	let guard = 0;
	do {
		state.turn += 1;
		if (state.turn >= state.order.length) {
			state.turn = 0;
			state.round += 1;
		}
		guard += 1;
	} while (!currentUnit(state) && guard < state.order.length + 2);
	beginTurn(state);
	check();
}
function skillOf(id) {
	return SKILLS[id];
}
function canUse(u, skill, hasRaku) {
	if (u.ap < skill.ap) return false;
	if (u.trans < skill.trans) return false;
	if (u.yoki < skill.yoki) return false;
	if (skill.id === "drop" && !hasRaku) return false;
	if (skill.id === "ripple" && !u.parts.some((p) => p.zone === "ripple" && p.hp > 0)) return false;
	return true;
}
function moveCost(state, hex) {
	const t = state.terrain[hexKey(hex)] ?? "grass";
	if (t === "water") return 99;
	if (t === "mud") return 2;
	return 1;
}
function legalMoves(state, unitId) {
	const u = state.units.find((x) => x.id === unitId);
	if (!u || u.dead) return [];
	const occ = occupiedMap(state, u.id);
	const start = coreHex(u);
	const budget = Math.min(u.ap, derived(u.stats).move);
	const out = [];
	const seen = /* @__PURE__ */ new Map();
	const q = [{
		h: start,
		c: 0
	}];
	seen.set(hexKey(start), 0);
	while (q.length) {
		const cur = q.shift();
		for (const n of hexNeighbors(cur.h)) {
			if (!inBounds(n, state.cols, state.rows)) continue;
			const cost = cur.c + moveCost(state, n);
			if (cost > budget) continue;
			const k = hexKey(n);
			if ((seen.get(k) ?? 99) <= cost) continue;
			if (occ.has(k)) continue;
			seen.set(k, cost);
			out.push(n);
			q.push({
				h: n,
				c: cost
			});
		}
	}
	return out;
}
function zoneFor(state, u, skill, target) {
	const from = coreHex(u);
	const face = facingToward(from, target);
	switch (skill.shape) {
		case "self": return [from];
		case "single": return [target];
		case "line": {
			const len = skill.length ?? skill.range;
			return hexLine(from, target).slice(1, len + 1).filter((h) => inBounds(h, state.cols, state.rows));
		}
		case "cone": return hexCone(from, face, skill.range).filter((h) => inBounds(h, state.cols, state.rows));
		case "blast": return hexDisc(target, skill.range).filter((h) => inBounds(h, state.cols, state.rows));
		case "ring": return hexRing(from, skill.range).filter((h) => inBounds(h, state.cols, state.rows));
		case "sweep": return hexSweep(from, face).filter((h) => inBounds(h, state.cols, state.rows));
		case "ripple": return hexRing(from, 1).filter((h) => inBounds(h, state.cols, state.rows));
		case "leap": return [target];
		default: return [target];
	}
}
function legalTargets(state, unitId, skillId) {
	const u = state.units.find((x) => x.id === unitId);
	const skill = SKILLS[skillId];
	if (!u || !skill) return [];
	const from = coreHex(u);
	if (skill.self || skill.shape === "self" || skill.shape === "ripple") return [from];
	if (skill.shape === "leap") {
		const occ = occupiedMap(state, u.id);
		return hexDisc(from, skill.range).filter((h) => {
			if (hexEq(h, from)) return false;
			if (!inBounds(h, state.cols, state.rows)) return false;
			if ((state.terrain[hexKey(h)] ?? "grass") === "water") return false;
			return !occ.has(hexKey(h));
		});
	}
	const cells = [];
	for (let q = 0; q < state.cols; q++) for (let r = 0; r < state.rows; r++) {
		const h = {
			q,
			r
		};
		const d = hexDistance(from, h);
		if (d < 1 || d > skill.range) continue;
		if (skill.shape === "single" || skill.shape === "blast") cells.push(h);
		else cells.push(h);
	}
	return cells;
}
function attrOf(u, a) {
	return u.stats[a];
}
function resolveHit(rng, atk, def, skill, cover) {
	const pa = attrOf(atk, skill.pa);
	const pd = attrOf(def, skill.pd);
	const scale = 1 + .25 * Math.max(-4, Math.min(8, pa - pd));
	const chance = .55 + (derived(atk.stats).hit + (skill.aimed ? -4 : 0) - derived(def.stats).dodge) * .03;
	const roll = rng.next();
	if (skill.unblockable) return {
		kind: "solid",
		scale
	};
	if (roll > chance + .15) return {
		kind: "miss",
		scale
	};
	if (roll > chance) return {
		kind: "glance",
		scale
	};
	if ((def.statuses.some((s) => (s.guard ?? 0) > 0) || cover) && rng.chance(.55)) return {
		kind: "blocked",
		scale
	};
	return {
		kind: "solid",
		scale
	};
}
function applyDamageToUnit(state, target, amount, zone, aimed, rng) {
	const placed = placeFootprint(target.footprint, target.origin, target.facing);
	const hitIdx = [];
	placed.forEach((h, i) => {
		const part = target.parts.find((p) => p.hexIndex === i);
		if (part && part.hp <= 0) return;
		if (zone.some((z) => hexEq(z, h))) hitIdx.push(i);
	});
	if (!hitIdx.length) return;
	const coreHit = hitIdx.includes(target.coreIndex);
	let dmg = Math.round(amount * (coreHit ? 1 : .5));
	const focus = aimed && hitIdx.length ? hitIdx.includes(target.coreIndex) ? target.coreIndex : hitIdx[0] : null;
	if (focus != null) {
		const part = target.parts.find((p) => p.hexIndex === focus);
		if (part) {
			part.hp = Math.max(0, part.hp - dmg);
			if (part.hp === 0 && target.parts.length > 1) {
				pushLog(state, "sever", `${target.name}'s ${part.name} is carved off.`);
				if (part.zone) target.skills = target.skills.filter((s) => SKILLS[s]?.id !== part.zone);
			}
		}
	} else {
		const share = Math.max(1, Math.round(dmg / hitIdx.length));
		for (const i of hitIdx) {
			const part = target.parts.find((p) => p.hexIndex === i);
			if (!part) continue;
			part.hp = Math.max(0, part.hp - share);
			if (part.hp === 0 && target.parts.length > 1) pushLog(state, "sever", `${target.name}'s ${part.name} is carved off.`);
		}
	}
	target.hp = target.parts.reduce((s, p) => s + p.hp, 0);
	if (target.hp <= 0) {
		target.dead = true;
		target.hp = 0;
		pushLog(state, "death", `${target.name} falls.`);
	}
}
function applyZone(state, zone, power, pa, pd, attackerId, aimed, unblockable) {
	const atk = state.units.find((u) => u.id === attackerId);
	if (!atk) return;
	const rng = new Rng(state.seed + state.round * 31 + state.turn * 7 + power);
	const dummySkill = {
		...SKILLS.cut,
		pa,
		pd,
		aimed,
		unblockable
	};
	const hitIds = /* @__PURE__ */ new Set();
	for (const h of zone) for (const u of state.units) {
		if (u.dead || u.id === attackerId) continue;
		if (u.side === atk.side && dummySkill) {}
		if (liveCells(u).some((c) => hexEq(c, h))) hitIds.add(u.id);
	}
	for (const id of hitIds) {
		const def = state.units.find((u) => u.id === id);
		if (!def) continue;
		const { kind, scale } = resolveHit(rng, atk, def, dummySkill, liveCells(def).some((c) => zone.some((z) => hexEq(z, c)) && state.terrain[hexKey(c)] === "ruin"));
		const transMul = 1 + atk.trans / 200;
		const base = power * scale * transMul;
		const dmg = kind === "miss" ? 0 : kind === "glance" ? Math.round(base * .2) : kind === "blocked" ? Math.round(base * rng.next() * .3) : Math.round(base);
		if (kind === "miss") pushLog(state, "miss", `${atk.name} misses ${def.name}.`);
		else if (kind === "blocked") pushLog(state, "hit", `${def.name} catches ${atk.name}'s blow.`);
		else pushLog(state, "hit", `${atk.name} → ${def.name}: ${kind} ${dmg}${aimed ? " (aimed)" : ""}.`);
		if (dmg > 0) applyDamageToUnit(state, def, dmg, zone, aimed, rng);
	}
}
function act(state, action, opts) {
	if (state.over) return state;
	const u = currentUnit(state);
	if (!u) return state;
	const next = structuredClone(state);
	const actor = next.units.find((x) => x.id === u.id);
	if (action.type === "raise") {
		if (actor.raisedTransThisTurn || actor.ap < 0) return state;
		actor.trans = Math.min(100, actor.trans + 16);
		actor.raisedTransThisTurn = true;
		pushLog(next, "trans", `${actor.name} opens the bar (${actor.trans}).`);
		if (actor.trans >= 100) pushLog(next, "trans", `${actor.name} is at the edge.`);
		return next;
	}
	if (action.type === "wait") {
		actor.ap = 0;
		actor.trans = Math.max(0, actor.trans - 4);
		pushLog(next, "info", `${actor.name} waits.`);
		advanceTurn(next);
		return next;
	}
	if (action.type === "move") {
		if (!legalMoves(next, actor.id).some((h) => hexEq(h, action.hex))) return state;
		const cost = Math.min(actor.ap, Math.max(1, hexDistance(coreHex(actor), action.hex)));
		actor.facing = facingToward(coreHex(actor), action.hex);
		const delta = {
			q: action.hex.q - coreHex(actor).q,
			r: action.hex.r - coreHex(actor).r
		};
		actor.origin = {
			q: actor.origin.q + delta.q,
			r: actor.origin.r + delta.r
		};
		actor.ap = Math.max(0, actor.ap - Math.max(1, cost));
		pushLog(next, "info", `${actor.name} steps.`);
		if (actor.ap <= 0) advanceTurn(next);
		return next;
	}
	const skill = SKILLS[action.skillId];
	if (!skill || !canUse(actor, skill, !!opts?.hasRaku)) return state;
	if (!legalTargets(next, actor.id, skill.id).some((h) => hexEq(h, action.hex))) return state;
	actor.facing = facingToward(coreHex(actor), action.hex);
	actor.ap -= skill.ap;
	actor.yoki -= skill.yoki;
	if (skill.transDelta) actor.trans = Math.max(0, Math.min(100, actor.trans + skill.transDelta));
	if (skill.heal) {
		const heal = skill.heal;
		actor.parts.forEach((p) => {
			if (p.hp > 0) p.hp = Math.min(p.maxHp, p.hp + Math.round(heal / actor.parts.length));
			else if (skill.id === "regen") p.hp = Math.min(p.maxHp, Math.round(p.maxHp * .4));
		});
		actor.hp = actor.parts.reduce((s, p) => s + p.hp, 0);
		pushLog(next, "info", `${actor.name} knits flesh (+${heal}).`);
	}
	if (skill.guard) actor.statuses.push({
		id: "guard",
		name: "Guard",
		turns: 2,
		guard: skill.guard
	});
	if (skill.telegraph) {
		actor.statuses.push({
			id: "read",
			name: "Read Energy",
			turns: 3,
			telegraph: true
		});
		for (const e of living(next, "enemy")) e.nextHint = pickAiSkill(next, e)?.name ?? "Advance";
		pushLog(next, "info", `${actor.name} reads the field.`);
	}
	if (skill.shape === "leap" && skill.move) {
		if (!occupiedMap(next, actor.id).has(hexKey(action.hex))) {
			if (skill.afterimage) actor.statuses.push({
				id: "after",
				name: "Afterimage",
				turns: 2,
				afterimage: { ...coreHex(actor) }
			});
			actor.origin = { ...action.hex };
			pushLog(next, "info", `${actor.name} is already gone.`);
		}
	}
	if (skill.shape === "ripple") {
		next.zones.push({
			id: `rip-${next.round}-${actor.id}`,
			kind: "ripple",
			sourceId: actor.id,
			center: { ...coreHex(actor) },
			radius: 1,
			maxRadius: 3,
			power: skill.power,
			pa: skill.pa
		});
		pushLog(next, "info", `${actor.name} starts a ripple.`);
	} else if (skill.power > 0) {
		const zone = zoneFor(next, actor, skill, action.hex);
		const strikes = skill.strikes ? derived(actor.stats).strikes : 1;
		for (let i = 0; i < strikes; i++) applyZone(next, zone, skill.power, skill.pa, skill.pd, actor.id, !!skill.aimed, !!skill.unblockable);
	}
	if (actor.ap <= 0) advanceTurn(next);
	if (living(next, "enemy").length === 0) next.over = "win";
	if (living(next, "player").length === 0) next.over = "lose";
	return next;
}
function pickAiSkill(state, u) {
	return u.skills.map((id) => SKILLS[id]).filter((s) => !!s && canUse(u, s, false) && s.power > 0).sort((a, b) => b.power - a.power)[0];
}
function runAi(state) {
	let cur = state;
	let guard = 0;
	while (!cur.over && currentUnit(cur)?.side === "enemy" && guard < 24) {
		guard += 1;
		const u = currentUnit(cur);
		if (!u) break;
		const foes = living(cur, "player");
		if (!foes.length) break;
		const from = coreHex(u);
		const nearest = [...foes].sort((a, b) => hexDistance(from, coreHex(a)) - hexDistance(from, coreHex(b)))[0];
		const skill = pickAiSkill(cur, u);
		if (skill && u.trans < skill.trans && !u.raisedTransThisTurn) {
			cur = act(cur, { type: "raise" });
			continue;
		}
		if (skill) {
			const targets = legalTargets(cur, u.id, skill.id);
			const foeCells = liveCells(nearest);
			const hit = targets.find((t) => {
				return zoneFor(cur, u, skill, t).some((h) => foeCells.some((c) => hexEq(c, h)));
			});
			if (hit) {
				cur = act(cur, {
					type: "skill",
					skillId: skill.id,
					hex: hit
				});
				continue;
			}
		}
		const moves = legalMoves(cur, u.id);
		if (moves.length) {
			const step = [...moves].sort((a, b) => hexDistance(a, coreHex(nearest)) - hexDistance(b, coreHex(nearest)))[0];
			if (hexDistance(step, coreHex(nearest)) < hexDistance(from, coreHex(nearest))) {
				cur = act(cur, {
					type: "move",
					hex: step
				});
				continue;
			}
		}
		cur = act(cur, { type: "wait" });
	}
	return cur;
}
function newWorld() {
	const locations = {};
	for (const loc of LOCATIONS) {
		let status = "quiet";
		if (loc.id === "dovra") status = "beacon";
		else if (loc.id === "paburo" || loc.id === "gonal" || loc.id === "pietra") status = "locked";
		else if (loc.id === "maw") status = "quiet";
		locations[loc.id] = {
			status,
			hoursLeft: loc.deadline ?? 0
		};
	}
	return {
		hours: 6,
		partyX: .28,
		partyY: .54,
		party: ["kira"],
		raku: false,
		rank: 47,
		karma: 0,
		authority: 40,
		ledger: {
			demons: 0,
			awakened: 0,
			silvers: 0,
			humans: 0,
			missions: 0
		},
		locations,
		flags: {},
		lastTown: void 0
	};
}
function dist01(ax, ay, bx, by) {
	const dx = ax - bx;
	const dy = ay - by;
	return Math.hypot(dx, dy);
}
function hoursForTravel(a, b) {
	return Math.max(3, Math.round(dist01(a.x, a.y, b.x, b.y) * 48));
}
function tickHours(world, hours) {
	const next = structuredClone(world);
	next.hours += hours;
	for (const loc of LOCATIONS) {
		const st = next.locations[loc.id];
		if (!st) continue;
		if (st.status === "beacon") {
			st.hoursLeft = Math.max(0, st.hoursLeft - hours);
			if (st.hoursLeft === 0) {
				st.status = "dead";
				next.karma -= 12;
			}
		}
	}
	return next;
}
function applyVictory(world, encounterId) {
	const enc = ENCOUNTERS[encounterId];
	const next = structuredClone(world);
	if (!enc) return next;
	const loc = LOCATIONS.find((l) => l.encounter === encounterId);
	if (loc) next.locations[loc.id] = {
		status: "cleared",
		hoursLeft: 0
	};
	next.ledger.missions += 1;
	if (encounterId.includes("ripple") || encounterId.includes("worm")) next.ledger.awakened += 1;
	else next.ledger.demons += encounterId === "paburo-nest" ? 3 : 2;
	if (enc.reward.karma) next.karma += enc.reward.karma;
	if (enc.reward.rank) next.rank = Math.max(1, next.rank + enc.reward.rank);
	if (enc.reward.raku) next.raku = true;
	if (enc.reward.recruit) {
		for (const id of enc.reward.recruit) if (!next.party.includes(id)) next.party.push(id);
	}
	next.flags[enc.reward.flag] = true;
	if (next.flags["dovra-cleared"] && next.locations.paburo?.status === "locked") next.locations.paburo = {
		status: "beacon",
		hoursLeft: 72
	};
	if (next.flags["paburo-cleared"] && next.locations.gonal?.status === "locked") next.locations.gonal = {
		status: "beacon",
		hoursLeft: 90
	};
	if (next.flags["gonal-cleared"] && next.locations.pietra?.status === "locked") next.locations.pietra = {
		status: "beacon",
		hoursLeft: 110
	};
	return next;
}
function nearestLocation(x, y, radius = .045) {
	let best;
	let bestD = radius;
	for (const loc of LOCATIONS) {
		const d = dist01(x, y, loc.x, loc.y);
		if (d < bestD) {
			bestD = d;
			best = loc;
		}
	}
	return best;
}
function clockLabel(hours) {
	return `Day ${Math.floor(hours / 24) + 1} · ${(hours % 24).toString().padStart(2, "0")}:00`;
}
var ctx = null;
function ac() {
	if (typeof window === "undefined") return null;
	if (!ctx) ctx = new (window.AudioContext || window.webkitAudioContext)();
	if (ctx.state === "suspended") ctx.resume();
	return ctx;
}
function unlockAudio() {
	ac();
}
function beep(freq, dur, type, gain = .04) {
	const c = ac();
	if (!c) return;
	const o = c.createOscillator();
	const g = c.createGain();
	o.type = type;
	o.frequency.value = freq;
	g.gain.value = gain;
	g.gain.exponentialRampToValueAtTime(1e-4, c.currentTime + dur);
	o.connect(g);
	g.connect(c.destination);
	o.start();
	o.stop(c.currentTime + dur);
}
var sfx = {
	ui: () => beep(520, .06, "square", .03),
	move: () => beep(180, .08, "triangle", .03),
	hit: () => beep(140, .12, "sawtooth", .05),
	miss: () => beep(240, .07, "sine", .025),
	trans: () => beep(90, .18, "sine", .05),
	win: () => {
		beep(440, .12, "square", .04);
		setTimeout(() => beep(660, .16, "square", .04), 90);
	},
	lose: () => beep(70, .3, "sawtooth", .05)
};
var SAVE_KEY = "wave.save.v1";
function blobOf(s) {
	return {
		v: 1,
		mode: s.mode === "title" ? "world" : s.mode,
		world: s.world,
		combat: s.combat,
		result: s.result
	};
}
function writeSave(s) {
	try {
		localStorage.setItem(SAVE_KEY, JSON.stringify(s));
	} catch {}
}
function readSave() {
	try {
		const raw = localStorage.getItem(SAVE_KEY);
		if (!raw) return null;
		const p = JSON.parse(raw);
		if (p.v !== 1) return null;
		return p;
	} catch {
		return null;
	}
}
var useGame = create((set, get) => ({
	mode: "title",
	world: newWorld(),
	combat: null,
	ui: {},
	keys: /* @__PURE__ */ new Set(),
	introStep: 0,
	boot: () => {
		const existing = readSave();
		if (existing) set({
			world: existing.world,
			combat: existing.combat,
			mode: "title",
			result: existing.result
		});
	},
	persist: () => {
		writeSave(blobOf(get()));
	},
	newHunt: () => {
		unlockAudio();
		sfx.ui();
		set({
			mode: "intro",
			world: newWorld(),
			combat: null,
			result: void 0,
			introStep: 0,
			ui: {}
		});
		get().persist();
	},
	continueHunt: () => {
		unlockAudio();
		const s = readSave();
		if (!s) return;
		sfx.ui();
		set({
			world: s.world,
			combat: s.combat,
			result: s.result,
			mode: s.combat ? "combat" : s.mode === "intro" ? "world" : s.mode
		});
	},
	setMode: (mode) => {
		set({ mode });
		get().persist();
	},
	moveParty: (dx, dy, dt) => {
		const { world, mode } = get();
		if (mode !== "world") return;
		const speed = .18;
		const nx = Math.min(.92, Math.max(.08, world.partyX + dx * speed * dt));
		const ny = Math.min(.88, Math.max(.1, world.partyY + dy * speed * dt));
		if (nx === world.partyX && ny === world.partyY) return;
		let next = {
			...world,
			partyX: nx,
			partyY: ny
		};
		next = tickHours(next, dt * 2.4);
		const loc = nearestLocation(nx, ny, .028);
		set({ world: next });
		if (loc && loc.id !== world.lastTown) get().enterLocation(loc.id);
	},
	travelTo: (id) => {
		const loc = locById(id);
		if (!loc) return;
		const { world } = get();
		const hours = hoursForTravel({
			x: world.partyX,
			y: world.partyY
		}, {
			x: loc.x,
			y: loc.y
		});
		set({ world: tickHours({
			...world,
			partyX: loc.x,
			partyY: loc.y,
			lastTown: void 0
		}, hours) });
		get().enterLocation(id);
	},
	enterLocation: (id) => {
		const loc = locById(id);
		if (!loc) return;
		sfx.ui();
		set({
			world: {
				...get().world,
				lastTown: id,
				partyX: loc.x,
				partyY: loc.y
			},
			mode: "town"
		});
		get().persist();
	},
	restTown: () => {
		const { world } = get();
		let next = tickHours(world, 8);
		next = {
			...next,
			karma: next.karma + 0
		};
		sfx.ui();
		set({ world: next });
		get().persist();
	},
	startEncounter: (id) => {
		const { world } = get();
		if (!ENCOUNTERS[id]) return;
		unlockAudio();
		const party = [...world.party];
		if (id === "paburo-nest") {
			if (!party.includes("vespera")) party.push("vespera");
			if (!party.includes("rhea")) party.push("rhea");
		}
		if (id === "pietra-worm" && !party.includes("nessa")) party.push("nessa");
		set({
			combat: runAi(createBattle(id, party)),
			mode: "combat",
			world: {
				...world,
				party
			},
			ui: {}
		});
		get().persist();
	},
	combatAct: (a) => {
		const { combat, world } = get();
		if (!combat || combat.over) return;
		let next = act(combat, a, { hasRaku: world.raku });
		if (a.type === "move") sfx.move();
		else if (a.type === "raise") sfx.trans();
		else sfx.hit();
		if (!next.over && next !== combat) next = runAi(next);
		if (next.over === "win") {
			sfx.win();
			const enc = ENCOUNTERS[next.id];
			const w = applyVictory(world, next.id);
			set({
				combat: next,
				world: w,
				result: {
					win: true,
					title: enc?.title ?? "Hunt ended",
					body: victoryCopy(next.id, w)
				},
				mode: "result",
				ui: {}
			});
			get().persist();
			return;
		}
		if (next.over === "lose") {
			sfx.lose();
			set({
				combat: next,
				result: {
					win: false,
					title: "The bar took you",
					body: "The Office will send someone else. They always do. The village will not remember your number."
				},
				mode: "result",
				ui: {}
			});
			get().persist();
			return;
		}
		set({
			combat: next,
			ui: {
				...get().ui,
				preview: void 0
			}
		});
	},
	setSkill: (id) => set({ ui: {
		...get().ui,
		selectedSkill: id
	} }),
	setHover: (h) => set({ ui: {
		...get().ui,
		hoverHex: h
	} }),
	holdKey: (code, down) => {
		const keys = new Set(get().keys);
		if (down) keys.add(code);
		else keys.delete(code);
		set({ keys });
	},
	dismissResult: () => {
		const { result } = get();
		if (!result?.win) {
			set({
				mode: "title",
				combat: null,
				result: void 0,
				world: newWorld()
			});
			get().persist();
			return;
		}
		set({
			mode: "world",
			combat: null,
			result: void 0
		});
		get().persist();
	}
}));
function victoryCopy(id, w) {
	if (id === "dovra-yoma") return `The well is only a well again. A boy named Raku will not leave the road. He can pull the bar down when you cannot. Rank still ${w.rank}.`;
	if (id === "paburo-nest") return `Vespera folds her cloak. Rhea shakes blood off an arm that is longer than it should be. They will walk with you. Gonal has gone quiet in the wrong way.`;
	if (id === "gonal-ripple") return `Ophel is meat on four empty hexes. The last ring fades. Pietra is calling every number that can still stand.`;
	if (id === "pietra-worm") return `The worm is a line of dead cells. Nessa puts a hand back on. The north will tell this story incorrectly.`;
	return "The board is empty.";
}
function hasSave() {
	return !!readSave();
}
var images = /* @__PURE__ */ new Map();
function img(src) {
	let im = images.get(src);
	if (!im) {
		im = new Image();
		im.crossOrigin = "anonymous";
		im.src = src;
		images.set(src, im);
	}
	return im;
}
function HexCanvas() {
	const ref = (0, import_react.useRef)(null);
	useGame((s) => s.combat);
	useGame((s) => s.ui);
	const combatAct = useGame((s) => s.combatAct);
	const setHover = useGame((s) => s.setHover);
	const setSkill = useGame((s) => s.setSkill);
	(0, import_react.useEffect)(() => {
		const canvas = ref.current;
		if (!canvas) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;
		let raf = 0;
		const bg = img("/art/battle-hamlet.jpg");
		const draw = () => {
			const st = useGame.getState();
			const battle = st.combat;
			const w = canvas.clientWidth;
			const h = canvas.clientHeight;
			const dpr = Math.min(2, window.devicePixelRatio || 1);
			if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) {
				canvas.width = Math.floor(w * dpr);
				canvas.height = Math.floor(h * dpr);
			}
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
			ctx.clearRect(0, 0, w, h);
			if (!battle) return;
			if (bg.complete) {
				const scale = Math.max(w / bg.naturalWidth, h / bg.naturalHeight);
				const dw = bg.naturalWidth * scale;
				const dh = bg.naturalHeight * scale;
				ctx.drawImage(bg, (w - dw) / 2, (h - dh) / 2, dw, dh);
			} else {
				ctx.fillStyle = "#161311";
				ctx.fillRect(0, 0, w, h);
			}
			ctx.fillStyle = "rgba(11,10,9,0.38)";
			ctx.fillRect(0, 0, w, h);
			const size = Math.min(w / (battle.cols * 1.85), h / (battle.rows * 1.75));
			const gridW = Math.sqrt(3) * size * (battle.cols + .5);
			const gridH = 3 / 2 * size * battle.rows + size;
			const ox = (w - gridW) / 2 + size;
			const oy = (h - gridH) / 2 + size * .8;
			const toPix = (hex) => {
				const p = axialToPixel(hex, size);
				return {
					x: ox + p.x,
					y: oy + p.y
				};
			};
			const actor = battle.units.find((u) => u.id === battle.order[battle.turn] && !u.dead);
			const moves = actor && actor.side === "player" ? legalMoves(battle, actor.id) : [];
			const skill = actor && st.ui.selectedSkill ? skillOf(st.ui.selectedSkill) : void 0;
			const targets = actor && skill && actor.side === "player" ? legalTargets(battle, actor.id, skill.id) : [];
			let preview = [];
			if (actor && skill && st.ui.hoverHex) preview = zoneFor(battle, actor, skill, st.ui.hoverHex);
			for (let q = 0; q < battle.cols; q++) for (let r = 0; r < battle.rows; r++) {
				const hex = {
					q,
					r
				};
				const { x, y } = toPix(hex);
				const terrain = battle.terrain[hexKey(hex)] ?? "grass";
				const pts = hexCorners(x, y, size - 1.2);
				ctx.beginPath();
				pts.forEach((p, i) => i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y));
				ctx.closePath();
				ctx.fillStyle = terrain === "water" ? "rgba(40,55,62,0.45)" : terrain === "mud" ? "rgba(70,52,36,0.42)" : terrain === "ruin" ? "rgba(70,68,64,0.4)" : "rgba(20,22,18,0.28)";
				ctx.fill();
				ctx.strokeStyle = "rgba(235,228,214,0.16)";
				ctx.lineWidth = 1;
				ctx.stroke();
			}
			const paintSet = (cells, fill, stroke) => {
				for (const hex of cells) {
					const { x, y } = toPix(hex);
					const pts = hexCorners(x, y, size - 1);
					ctx.beginPath();
					pts.forEach((p, i) => i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y));
					ctx.closePath();
					ctx.fillStyle = fill;
					ctx.fill();
					if (stroke) {
						ctx.strokeStyle = stroke;
						ctx.lineWidth = 2;
						ctx.stroke();
					}
				}
			};
			paintSet(moves, "rgba(200,204,212,0.16)", "rgba(200,204,212,0.45)");
			paintSet(targets, "rgba(154,36,48,0.10)");
			paintSet(preview, "rgba(154,36,48,0.38)", "rgba(235,228,214,0.7)");
			for (const z of battle.zones) {
				const ring = [];
				for (let q = 0; q < battle.cols; q++) for (let r = 0; r < battle.rows; r++) {
					const dq = q - z.center.q;
					const dr = r - z.center.r;
					if ((Math.abs(dq) + Math.abs(dq + dr) + Math.abs(dr)) / 2 === z.radius) ring.push({
						q,
						r
					});
				}
				paintSet(ring, "rgba(154,36,48,0.22)", "rgba(154,36,48,0.8)");
			}
			const sorted = [...battle.units].filter((u) => !u.dead).sort((a, b) => {
				const ac = coreHex(a);
				const bc = coreHex(b);
				return ac.r - bc.r || ac.q - bc.q;
			});
			for (const u of sorted) {
				const cells = liveCells(u);
				for (const c of cells) {
					const { x, y } = toPix(c);
					const pts = hexCorners(x, y, size - 2);
					ctx.beginPath();
					pts.forEach((p, i) => i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y));
					ctx.closePath();
					ctx.fillStyle = u.side === "player" ? "rgba(200,204,212,0.22)" : "rgba(154,36,48,0.22)";
					ctx.fill();
					ctx.strokeStyle = u.id === actor?.id ? "#ebe4d6" : u.color;
					ctx.lineWidth = u.id === actor?.id ? 2.4 : 1.4;
					ctx.stroke();
				}
				const { x, y } = toPix(coreHex(u));
				const spr = img(u.sprite ?? u.portrait);
				const ih = size * (cells.length > 1 ? 2.8 : 2.15);
				const iw = ih * .7;
				if (spr.complete && spr.naturalWidth) {
					ctx.save();
					ctx.shadowColor = "rgba(0,0,0,0.55)";
					ctx.shadowBlur = 16;
					ctx.drawImage(spr, x - iw / 2, y - ih * .78, iw, ih);
					ctx.restore();
				} else {
					ctx.beginPath();
					ctx.arc(x, y, size * .45, 0, Math.PI * 2);
					ctx.fillStyle = u.color;
					ctx.fill();
				}
				ctx.font = "600 11px Figtree, sans-serif";
				ctx.textAlign = "center";
				ctx.fillStyle = "#ebe4d6";
				ctx.fillText(u.name, x, y + size * .85);
				const ratio = u.hp / Math.max(1, u.maxHp);
				ctx.fillStyle = "rgba(11,10,9,0.7)";
				ctx.fillRect(x - 18, y + size * .92, 36, 4);
				ctx.fillStyle = ratio < .35 ? "#9a2430" : "#c8ccd4";
				ctx.fillRect(x - 18, y + size * .92, 36 * ratio, 4);
				if (u.nextHint) {
					ctx.fillStyle = "#c8ccd4";
					ctx.font = "500 10px Figtree, sans-serif";
					ctx.fillText(u.nextHint, x, y - size * 1.05);
				}
			}
			raf = requestAnimationFrame(draw);
		};
		raf = requestAnimationFrame(draw);
		return () => cancelAnimationFrame(raf);
	}, []);
	function hexAt(e) {
		const canvas = ref.current;
		const battle = useGame.getState().combat;
		if (!canvas || !battle) return;
		const rect = canvas.getBoundingClientRect();
		const w = rect.width;
		const h = rect.height;
		const size = Math.min(w / (battle.cols * 1.85), h / (battle.rows * 1.75));
		const gridW = Math.sqrt(3) * size * (battle.cols + .5);
		const gridH = 3 / 2 * size * battle.rows + size;
		const ox = (w - gridW) / 2 + size;
		const oy = (h - gridH) / 2 + size * .8;
		const hex = pixelToAxial(e.clientX - rect.left - ox, e.clientY - rect.top - oy, size);
		if (hex.q < 0 || hex.r < 0 || hex.q >= battle.cols || hex.r >= battle.rows) return;
		return hex;
	}
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)("canvas", {
		ref,
		className: "absolute inset-0 h-full w-full touch-none",
		onPointerMove: (e) => setHover(hexAt(e)),
		onPointerLeave: () => setHover(void 0),
		onPointerDown: (e) => {
			const hex = hexAt(e);
			const st = useGame.getState();
			const battle = st.combat;
			if (!hex || !battle || battle.over) return;
			const actor = battle.units.find((u) => u.id === battle.order[battle.turn] && !u.dead);
			if (!actor || actor.side !== "player") return;
			const skill = st.ui.selectedSkill ? skillOf(st.ui.selectedSkill) : void 0;
			if (skill) {
				combatAct({
					type: "skill",
					skillId: skill.id,
					hex
				});
				setSkill(void 0);
				return;
			}
			if (legalMoves(battle, actor.id).some((m) => hexEq(m, hex))) combatAct({
				type: "move",
				hex
			});
		}
	});
}
function WorldCanvas() {
	const ref = (0, import_react.useRef)(null);
	const world = useGame((s) => s.world);
	const travelTo = useGame((s) => s.travelTo);
	(0, import_react.useEffect)(() => {
		const canvas = ref.current;
		if (!canvas) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;
		const map = new Image();
		map.src = "/art/world-map.jpg";
		let raf = 0;
		let last = performance.now();
		const loop = (now) => {
			const dt = Math.min(.05, (now - last) / 1e3);
			last = now;
			const w = canvas.clientWidth;
			const h = canvas.clientHeight;
			const dpr = Math.min(2, window.devicePixelRatio || 1);
			if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) {
				canvas.width = Math.floor(w * dpr);
				canvas.height = Math.floor(h * dpr);
			}
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
			ctx.clearRect(0, 0, w, h);
			ctx.fillStyle = "#0b0a09";
			ctx.fillRect(0, 0, w, h);
			let dx = 0;
			let dy = 0;
			const held = useGame.getState().keys;
			if (held.has("KeyA") || held.has("ArrowLeft")) dx -= 1;
			if (held.has("KeyD") || held.has("ArrowRight")) dx += 1;
			if (held.has("KeyW") || held.has("ArrowUp")) dy -= 1;
			if (held.has("KeyS") || held.has("ArrowDown")) dy += 1;
			if (dx || dy) {
				const len = Math.hypot(dx, dy) || 1;
				useGame.getState().moveParty(dx / len, dy / len, dt);
			}
			const st = useGame.getState().world;
			if (map.complete && map.naturalWidth) {
				const iw = map.naturalWidth;
				const ih = map.naturalHeight;
				const sx = iw * .07;
				const sy = ih * .03;
				const sw = iw * .86;
				const sh = ih * .84;
				const scale = Math.max(w / sw, h / sh);
				const dw = sw * scale;
				const dh = sh * scale;
				const ox = (w - dw) / 2;
				const oy = (h - dh) / 2;
				ctx.drawImage(map, sx, sy, sw, sh, ox, oy, dw, dh);
				ctx.fillStyle = "rgba(11,10,9,0.12)";
				ctx.fillRect(0, 0, w, h);
				const toScreen = (lx, ly) => ({
					x: ox + (lx * iw - sx) / sw * dw,
					y: oy + (ly * ih - sy) / sh * dh
				});
				for (const loc of LOCATIONS) {
					const ls = st.locations[loc.id];
					const { x, y } = toScreen(loc.x, loc.y);
					const status = ls?.status ?? "quiet";
					if (status === "locked") continue;
					if (status === "beacon") {
						const pulse = 10 + Math.sin(now / 280) * 4;
						ctx.beginPath();
						ctx.arc(x, y, pulse + 10, 0, Math.PI * 2);
						ctx.fillStyle = "rgba(154,36,48,0.18)";
						ctx.fill();
					}
					ctx.beginPath();
					ctx.arc(x, y, status === "beacon" ? 7 : 5, 0, Math.PI * 2);
					ctx.fillStyle = status === "beacon" ? "#9a2430" : status === "dead" ? "#3a322c" : status === "cleared" ? "#c8ccd4" : "#8d8578";
					ctx.fill();
					ctx.strokeStyle = "rgba(235,228,214,0.45)";
					ctx.lineWidth = 1;
					ctx.stroke();
					ctx.font = "600 12px Figtree, sans-serif";
					ctx.fillStyle = "#ebe4d6";
					ctx.textAlign = "center";
					ctx.fillText(loc.name, x, y - 12);
				}
				const { x: px, y: py } = toScreen(st.partyX, st.partyY);
				ctx.beginPath();
				ctx.arc(px, py, 9, 0, Math.PI * 2);
				ctx.fillStyle = "#ebe4d6";
				ctx.fill();
				ctx.strokeStyle = "#0b0a09";
				ctx.lineWidth = 2;
				ctx.stroke();
				const lead = WARRIORS[st.party[0] ?? "kira"];
				ctx.font = "600 11px Figtree, sans-serif";
				ctx.fillStyle = "#ebe4d6";
				ctx.fillText(lead?.name ?? "Kira", px, py + 22);
			}
			raf = requestAnimationFrame(loop);
		};
		raf = requestAnimationFrame(loop);
		return () => cancelAnimationFrame(raf);
	}, []);
	(0, import_react.useEffect)(() => {
		const onKey = (e, down) => {
			if ([
				"KeyW",
				"KeyA",
				"KeyS",
				"KeyD",
				"ArrowUp",
				"ArrowDown",
				"ArrowLeft",
				"ArrowRight"
			].includes(e.code)) {
				e.preventDefault();
				useGame.getState().holdKey(e.code, down);
			}
		};
		const down = (e) => onKey(e, true);
		const up = (e) => onKey(e, false);
		const clear = () => {
			[
				"KeyW",
				"KeyA",
				"KeyS",
				"KeyD",
				"ArrowUp",
				"ArrowDown",
				"ArrowLeft",
				"ArrowRight"
			].forEach((c) => useGame.getState().holdKey(c, false));
		};
		window.addEventListener("keydown", down);
		window.addEventListener("keyup", up);
		window.addEventListener("blur", clear);
		return () => {
			window.removeEventListener("keydown", down);
			window.removeEventListener("keyup", up);
			window.removeEventListener("blur", clear);
		};
	}, []);
	(0, import_react.useEffect)(() => {
		window.__controlsTest = {
			getYaw: () => 0,
			getSpeed: () => useGame.getState().keys.size ? 1 : 0,
			getX: () => useGame.getState().world.partyX,
			getY: () => useGame.getState().world.partyY,
			setKeys: (codes) => {
				useGame.getState().keys.forEach((c) => useGame.getState().holdKey(c, false));
				codes.forEach((c) => useGame.getState().holdKey(c, true));
			}
		};
		return () => {
			delete window.__controlsTest;
		};
	}, []);
	function onClick(e) {
		const canvas = ref.current;
		if (!canvas) return;
		const rect = canvas.getBoundingClientRect();
		const w = rect.width;
		const h = rect.height;
		const iw = 1792;
		const ih = 1008;
		const sx = iw * .07;
		const sy = ih * .03;
		const sw = iw * .86;
		const sh = ih * .84;
		const scale = Math.max(w / sw, h / sh);
		const dw = sw * scale;
		const dh = sh * scale;
		const ox = (w - dw) / 2;
		const oy = (h - dh) / 2;
		const hit = nearestLocation((e.clientX - rect.left - ox) / dw * (sw / iw) + sx / iw, (e.clientY - rect.top - oy) / dh * (sh / ih) + sy / ih, .05);
		if (hit && world.locations[hit.id]?.status !== "locked") travelTo(hit.id);
	}
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)("canvas", {
		ref,
		className: "absolute inset-0 h-full w-full touch-none",
		onClick
	});
}
function CombatHud() {
	const combat = useGame((s) => s.combat);
	const world = useGame((s) => s.world);
	const ui = useGame((s) => s.ui);
	const setSkill = useGame((s) => s.setSkill);
	const combatAct = useGame((s) => s.combatAct);
	if (!combat) return null;
	const actor = currentUnit(combat);
	const mine = actor?.side === "player";
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "pointer-events-none absolute inset-0 z-10 flex flex-col justify-between p-3 sm:p-4",
		children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
			className: "flex items-start justify-between gap-3 pr-24",
			children: /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "pointer-events-auto max-w-md rounded-lg border border-line bg-fog px-4 py-3",
				children: [
					/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("p", {
						className: "font-display text-xs tracking-[0.22em] text-dust uppercase",
						children: [
							combat.title,
							" · Round ",
							combat.round
						]
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-1 text-xs leading-5 text-ash/80",
						children: combat.briefing
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("ol", {
						className: "mt-2 flex flex-wrap gap-1",
						children: combat.order.map((id) => combat.units.find((u) => u.id === id)).filter(Boolean).map((u) => /* @__PURE__ */ (0, import_jsx_runtime.jsx)("li", {
							className: `rounded-sm px-1.5 py-0.5 text-[10px] ${u.dead ? "text-dust/40 line-through" : u.id === actor?.id ? "bg-steel text-ink" : "bg-ink/40 text-dust"}`,
							children: u.name
						}, u.id))
					})
				]
			})
		}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
			className: "flex flex-col gap-3 lg:flex-row lg:items-end",
			children: [
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: "pointer-events-auto max-h-36 overflow-y-auto rounded-lg border border-line bg-fog px-3 py-2 text-[11px] leading-5 text-dust lg:w-72",
					children: combat.log.slice(0, 8).map((l, i) => /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: l.kind === "death" || l.kind === "sever" ? "text-blood" : "",
						children: l.text
					}, `${l.t}-${i}`))
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: "pointer-events-auto min-w-0 flex-1 rounded-xl border border-line bg-fog p-3",
					children: actor && /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_jsx_runtime.Fragment, { children: [
						/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
							className: "flex items-center gap-3",
							children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
								src: actor.portrait,
								alt: "",
								className: "h-14 w-10 rounded-sm object-cover"
							}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
								className: "min-w-0 flex-1",
								children: [
									/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
										className: "flex items-baseline justify-between gap-2",
										children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
											className: "font-display text-xl leading-none",
											children: actor.name
										}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("p", {
											className: "text-[11px] text-dust tabular-nums",
											children: [
												"AP ",
												actor.ap,
												"/",
												actor.maxAp
											]
										})]
									}),
									/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
										className: "text-[11px] text-dust",
										children: actor.title
									}),
									/* @__PURE__ */ (0, import_jsx_runtime.jsx)(Meter, {
										label: "Flesh",
										value: actor.hp,
										max: actor.maxHp,
										tone: "steel"
									}),
									/* @__PURE__ */ (0, import_jsx_runtime.jsx)(Meter, {
										label: "Yoki",
										value: actor.yoki,
										max: actor.maxYoki,
										tone: "steel"
									}),
									/* @__PURE__ */ (0, import_jsx_runtime.jsx)(Meter, {
										label: "Trans",
										value: actor.trans,
										max: 100,
										tone: "blood"
									})
								]
							})]
						}),
						actor.parts.length > 1 && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
							className: "mt-2 flex flex-wrap gap-1",
							children: actor.parts.map((p) => /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("span", {
								className: `rounded-sm border px-1.5 py-0.5 text-[10px] ${p.hp <= 0 ? "border-blood/40 text-blood line-through" : "border-line text-dust"}`,
								children: [
									p.name,
									" ",
									p.hp,
									"/",
									p.maxHp
								]
							}, p.id))
						}),
						mine && /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_jsx_runtime.Fragment, { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
							className: "mt-2 text-[11px] text-dust",
							children: "Pale hexes are steps. Choose a technique, then a cell. Raise the bar to unlock the rest."
						}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
							className: "mt-3 flex flex-wrap gap-1.5",
							children: [
								actor.skills.map((id) => {
									const s = SKILLS[id];
									if (!s) return null;
									const ok = canUse(actor, s, world.raku);
									const on = ui.selectedSkill === id;
									return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("button", {
										type: "button",
										disabled: !ok,
										onClick: () => {
											if (s.self || s.shape === "self" || s.shape === "ripple") {
												const core = actor.origin;
												combatAct({
													type: "skill",
													skillId: id,
													hex: core
												});
												setSkill(void 0);
												return;
											}
											setSkill(on ? void 0 : id);
										},
										className: `rounded-md border px-2.5 py-1.5 text-left text-[11px] ${on ? "border-steel bg-steel text-ink" : ok ? "border-line bg-raised text-ash hover:border-steel/40" : "border-transparent bg-ink/30 text-dust/50"}`,
										children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
											className: "block font-medium",
											children: s.name
										}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("span", {
											className: "text-[10px] opacity-70",
											children: [
												s.ap,
												" AP",
												s.trans ? ` · T${s.trans}` : ""
											]
										})]
									}, id);
								}),
								/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
									type: "button",
									onClick: () => combatAct({ type: "raise" }),
									disabled: actor.raisedTransThisTurn,
									className: "rounded-md border border-blood/40 px-2.5 py-1.5 text-[11px] text-ash hover:bg-blood/20 disabled:opacity-40",
									children: "Raise bar"
								}),
								/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
									type: "button",
									onClick: () => combatAct({ type: "wait" }),
									className: "rounded-md border border-line px-2.5 py-1.5 text-[11px] text-dust hover:text-ash",
									children: "Wait"
								})
							]
						})] }),
						!mine && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
							className: "mt-2 text-xs text-dust",
							children: "The other side is moving."
						})
					] })
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: "pointer-events-none hidden w-36 flex-col gap-1 lg:flex",
					children: living(combat).map((u) => /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
						className: "flex items-center gap-2 rounded-md border border-line bg-fog px-2 py-1",
						children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
							className: "h-2 w-2 rounded-full",
							style: { background: u.side === "player" ? "#c8ccd4" : "#9a2430" }
						}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
							className: "truncate text-[11px]",
							children: u.name
						})]
					}, u.id))
				})
			]
		})]
	});
}
function Meter({ label, value, max, tone }) {
	const pct = Math.max(0, Math.min(100, value / Math.max(1, max) * 100));
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "mt-1 flex items-center gap-2",
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
				className: "w-10 text-[10px] tracking-wide text-dust uppercase",
				children: label
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
				className: "h-1.5 flex-1 rounded-full bg-ink/70",
				children: /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: `h-full rounded-full ${tone === "blood" ? "bg-blood" : "bg-steel"}`,
					style: { width: `${pct}%` }
				})
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
				className: "w-10 text-right text-[10px] text-dust tabular-nums",
				children: Math.round(value)
			})
		]
	});
}
var chapters = [
	{
		t: "The brief",
		b: "WAVE is the 2007 notes made playable. An island like Fallout 2. Hunts on a hex grid. Warriors occupy one cell. Awakened occupy many. Damage is zonal — you cut a limb off the board."
	},
	{
		t: "What the notes kept",
		b: "Trans-meter. Discrete hits (miss / glance / blocked / solid). Scale 1 + 0.25×clamp(Pa−Pd). Perception auras. Time-dying villages. Rank, karma, a single human companion. Learning by watching and bleeding."
	},
	{
		t: "What changed",
		b: "The notes wanted pause-RT. The current brief wants turn-based hex. Pause-RT is archived, not deleted. The roster is the television series with legally distinct names."
	},
	{
		t: "The bar",
		b: "Raise it at any time. Skills unlock in bands: 20 aimed and read, 40 Flash and Phantom, 60 Spiral, 80 wings. At 90 the turn can vanish. At 100 you are the encounter."
	},
	{
		t: "Zonal bosses",
		b: "Ophel is four hexes: core, torso, tail, cutting arm. A zone that hits a limb does half and can sever it. Sever the tail and Ripple Edge dies. Only the core left is faster and smaller."
	},
	{
		t: "Blade",
		b: "Native target is Blade (Rust): wave-sim as the rules crate, blade-engine for the island and hunt scenes, egui for the tavern. This preview is the rules-accurate web slice — the sandbox cannot open Vulkan."
	}
];
function CodexView() {
	const setMode = useGame((s) => s.setMode);
	const back = useGame((s) => s.world).hours > 6 ? "world" : "title";
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
		className: "absolute inset-0 overflow-y-auto bg-ink",
		children: /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
			className: "mx-auto max-w-3xl px-5 py-10",
			children: [
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
					type: "button",
					onClick: () => setMode(back),
					className: "text-xs tracking-[0.2em] text-dust uppercase hover:text-ash",
					children: "Close"
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h1", {
					className: "mt-4 font-display text-5xl font-semibold",
					children: "Codex"
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
					className: "mt-2 text-sm text-dust",
					children: "From the October notes, the sketches, and the series."
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: "mt-8 space-y-8",
					children: chapters.map((c) => /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("section", { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
						className: "font-display text-2xl",
						children: c.t
					}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-2 text-sm leading-7 text-ash/85",
						children: c.b
					})] }, c.t))
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
					className: "mt-12 font-display text-2xl",
					children: "Roster"
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: "mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4",
					children: Object.values(WARRIORS).map((w) => /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("figure", {
						className: "overflow-hidden rounded-lg border border-line",
						children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
							src: w.portrait,
							alt: w.name,
							className: "aspect-2/3 w-full object-cover"
						}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("figcaption", {
							className: "px-2 py-2",
							children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
								className: "text-sm",
								children: w.name
							}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
								className: "text-[11px] text-dust",
								children: w.title
							})]
						})]
					}, w.id))
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
					className: "mt-12 font-display text-2xl",
					children: "Techniques"
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("ul", {
					className: "mt-4 divide-y divide-line border-y border-line",
					children: Object.values(SKILLS).filter((s) => ![
						"claw",
						"lunge",
						"slam",
						"whip"
					].includes(s.id)).map((s) => /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("li", {
						className: "py-3",
						children: [/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("p", {
							className: "text-sm font-medium",
							children: [s.name, /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("span", {
								className: "ml-2 text-[11px] font-normal text-dust",
								children: [
									"T",
									s.trans,
									" · ",
									s.shape
								]
							})]
						}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
							className: "mt-1 text-xs leading-5 text-dust",
							children: s.blurb
						})]
					}, s.id))
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
					className: "mt-12 font-display text-2xl",
					children: "Kenney — pull these"
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
					className: "mt-2 text-sm leading-7 text-dust",
					children: "From the All-in-1 pack: Hexagon Pack + Base + Buildings; Isometric Nature, Medieval Town, Miniature Dungeon; RPG Tileset; UI Pack + RPG extension; Input Prompts; Cursor Pack; Particle Pack; Rune Pack; RPG / Interface audio. For Blade later: Nature Kit, Castle Kit, Graveyard Kit, Modular Dungeon / Cave. Leave the tanks and spaceships in the box."
				})
			]
		})
	});
}
function ResultView() {
	const result = useGame((s) => s.result);
	const dismiss = useGame((s) => s.dismissResult);
	if (!result) return null;
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "absolute inset-0",
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
				src: result.win ? "/art/title.jpg" : "/art/ophel.jpg",
				alt: "",
				className: "absolute inset-0 h-full w-full object-cover opacity-40"
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", { className: "absolute inset-0 bg-ink/75" }),
			/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "relative mx-auto flex min-h-dvh max-w-lg flex-col justify-center px-6",
				children: [
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "font-display text-xs tracking-[0.28em] text-dust uppercase",
						children: result.win ? "The board is empty" : "Hunt failed"
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
						className: "mt-2 font-display text-4xl font-semibold",
						children: result.title
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-4 text-sm leading-7 text-ash/85",
						children: result.body
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
						type: "button",
						onClick: dismiss,
						className: "mt-8 w-fit rounded-md bg-steel px-5 py-3 text-sm font-medium text-ink hover:bg-ash",
						children: result.win ? "Return to the island" : "The Office sends another"
					})
				]
			})
		]
	});
}
function TitleScreen({ hasSave }) {
	const newHunt = useGame((s) => s.newHunt);
	const continueHunt = useGame((s) => s.continueHunt);
	const setMode = useGame((s) => s.setMode);
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "absolute inset-0",
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
				src: "/art/title.jpg",
				alt: "",
				className: "absolute inset-0 h-full w-full object-cover"
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", { className: "absolute inset-0 bg-linear-to-r from-ink via-ink/70 to-ink/20" }),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", { className: "absolute inset-0 bg-linear-to-t from-ink via-transparent to-ink/30" }),
			/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "relative flex min-h-dvh flex-col justify-end px-6 pt-16 pb-10 sm:justify-center sm:px-12",
				children: [
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "font-display text-xs tracking-[0.38em] text-dust uppercase",
						children: "A hunt across the island"
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h1", {
						className: "mt-3 font-display text-[clamp(4.2rem,14vw,8rem)] leading-[0.85] font-semibold tracking-[-0.03em]",
						children: "WAVE"
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-5 max-w-md text-sm leading-7 text-ash/80 sm:text-base",
						children: "Silver-eyed warriors. A Fallout map. Hex hunts. Multi-cell demons you carve apart. The bar in your chest is not a metaphor."
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
						className: "mt-8 flex max-w-sm flex-col gap-2",
						children: [
							/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
								type: "button",
								onClick: newHunt,
								className: "rounded-md bg-steel px-5 py-3 text-sm font-medium text-ink transition hover:bg-ash",
								children: "New hunt"
							}),
							hasSave && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
								type: "button",
								onClick: continueHunt,
								className: "rounded-md border border-line bg-surface/70 px-5 py-3 text-sm font-medium text-ash hover:border-steel/40",
								children: "Continue"
							}),
							/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
								type: "button",
								onClick: () => setMode("codex"),
								className: "rounded-md px-5 py-3 text-sm text-dust hover:text-ash",
								children: "Design codex"
							})
						]
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-8 max-w-sm text-[11px] leading-5 text-dust",
						children: "WASD walks the island. Click a pin to travel. In a hunt: click a hex to step, pick a technique, click a cell to cut. Raise the bar when you must."
					})
				]
			})
		]
	});
}
function TownView() {
	const world = useGame((s) => s.world);
	const setMode = useGame((s) => s.setMode);
	const restTown = useGame((s) => s.restTown);
	const startEncounter = useGame((s) => s.startEncounter);
	const loc = locById(world.lastTown ?? "dovra");
	if (!loc) return null;
	const enc = loc.encounter ? ENCOUNTERS[loc.encounter] : void 0;
	const st = world.locations[loc.id];
	const huntReady = !!enc && !!st && (st.status === "beacon" || st.status === "dead");
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "absolute inset-0",
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
				src: loc.kind === "village" || loc.kind === "city" ? "/art/tavern.jpg" : "/art/battle-hamlet.jpg",
				alt: "",
				className: "absolute inset-0 h-full w-full object-cover"
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", { className: "absolute inset-0 bg-ink/70" }),
			/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "relative mx-auto flex min-h-dvh max-w-3xl flex-col justify-end px-5 py-8 sm:justify-center",
				children: [
					/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("p", {
						className: "font-display text-xs tracking-[0.28em] text-dust uppercase",
						children: [
							loc.region,
							" · ",
							clockLabel(world.hours)
						]
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
						className: "mt-2 font-display text-5xl font-semibold",
						children: loc.name
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-3 max-w-xl text-sm leading-7 text-ash/85",
						children: loc.blurb
					}),
					st?.status === "beacon" && /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("p", {
						className: "text-blood mt-2 text-sm",
						children: [st.hoursLeft, " hours before this pin goes black."]
					}),
					st?.status === "dead" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-2 text-sm text-dust",
						children: "The beacon died. This is a nest now."
					}),
					st?.status === "cleared" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-2 text-sm text-dust",
						children: "The hunt here is finished. They still flinch when you pass."
					}),
					world.raku && loc.id === "dovra" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-3 text-sm text-steel",
						children: "Raku waits by the well. Trans Drop is yours while he lives."
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
						className: "mt-6 flex flex-wrap gap-2",
						children: [
							huntReady && enc && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
								type: "button",
								onClick: () => startEncounter(enc.id),
								className: "rounded-md bg-steel px-5 py-3 text-sm font-medium text-ink hover:bg-ash",
								children: "Begin hunt"
							}),
							/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
								type: "button",
								onClick: restTown,
								className: "rounded-md border border-line bg-surface/80 px-5 py-3 text-sm hover:border-steel/40",
								children: "Rest until morning"
							}),
							/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
								type: "button",
								onClick: () => setMode("world"),
								className: "rounded-md px-5 py-3 text-sm text-dust hover:text-ash",
								children: "Back to the road"
							})
						]
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
						className: "mt-8 flex gap-2",
						children: world.party.map((id) => {
							const w = WARRIORS[id];
							if (!w) return null;
							return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
								className: "w-20",
								children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
									src: w.portrait,
									alt: "",
									className: "aspect-2/3 w-full rounded-md object-cover"
								}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
									className: "mt-1 text-center text-[11px] text-dust",
									children: w.name
								})]
							}, id);
						})
					})
				]
			})
		]
	});
}
function WorldHud() {
	const world = useGame((s) => s.world);
	const travelTo = useGame((s) => s.travelTo);
	const setMode = useGame((s) => s.setMode);
	const beacons = LOCATIONS.filter((l) => world.locations[l.id]?.status === "beacon");
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_jsx_runtime.Fragment, { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("header", {
		className: "pointer-events-none absolute top-0 right-0 left-0 z-10 flex items-start justify-between p-4 pr-28",
		children: [/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
			className: "pointer-events-auto rounded-lg border border-line bg-fog px-4 py-3",
			children: [
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
					className: "font-display text-xs tracking-[0.24em] text-dust uppercase",
					children: "Island"
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
					className: "font-display text-2xl leading-none",
					children: clockLabel(world.hours)
				}),
				/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("p", {
					className: "mt-1 text-xs text-dust",
					children: [
						"Rank ",
						world.rank,
						" · Karma ",
						world.karma >= 0 ? "+" : "",
						world.karma
					]
				})
			]
		}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
			className: "pointer-events-auto hidden gap-2 sm:flex",
			children: world.party.map((id) => {
				const w = WARRIORS[id];
				if (!w) return null;
				return /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
					className: "overflow-hidden rounded-md border border-line",
					children: /* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
						src: w.portrait,
						alt: w.name,
						className: "h-14 w-10 object-cover"
					})
				}, id);
			})
		})]
	}), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("aside", {
		className: "pointer-events-auto absolute bottom-4 left-4 z-10 w-[min(100%-2rem,22rem)] rounded-lg border border-line bg-fog p-4",
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
				className: "font-display text-xs tracking-[0.22em] text-dust uppercase",
				children: "Beacons"
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("ul", {
				className: "mt-2 space-y-2",
				children: [beacons.length === 0 && /* @__PURE__ */ (0, import_jsx_runtime.jsx)("li", {
					className: "text-sm text-dust",
					children: "No village is screaming. Walk. The Maw is still dark."
				}), beacons.map((b) => {
					const st = world.locations[b.id];
					return /* @__PURE__ */ (0, import_jsx_runtime.jsx)("li", { children: /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("button", {
						type: "button",
						onClick: () => travelTo(b.id),
						className: "flex w-full items-center justify-between rounded-md px-1 py-1 text-left hover:bg-ash/5",
						children: [/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("span", { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
							className: "block text-sm font-medium",
							children: b.name
						}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
							className: "text-xs text-dust",
							children: b.region
						})] }), /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("span", {
							className: "font-display text-blood text-sm tabular-nums",
							children: [st.hoursLeft, "h"]
						})]
					}) }, b.id);
				})]
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "mt-3 flex gap-2",
				children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
					type: "button",
					onClick: () => setMode("codex"),
					className: "rounded-md border border-line px-3 py-1.5 text-xs text-dust hover:text-ash",
					children: "Codex"
				}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("button", {
					type: "button",
					onClick: () => setMode("title"),
					className: "rounded-md border border-line px-3 py-1.5 text-xs text-dust hover:text-ash",
					children: "Title"
				})]
			})
		]
	})] });
}
function GameApp() {
	const mode = useGame((s) => s.mode);
	const boot = useGame((s) => s.boot);
	const { isPending } = useCurrentUserState();
	const [saved, setSaved] = (0, import_react.useState)(false);
	(0, import_react.useEffect)(() => {
		boot();
		setSaved(hasSave());
	}, [boot]);
	(0, import_react.useEffect)(() => {
		if (mode === "title") setSaved(hasSave());
	}, [mode]);
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "relative h-dvh min-h-dvh overflow-hidden bg-ink text-ash",
		children: [
			mode === "title" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)(TitleScreen, { hasSave: saved }),
			mode === "intro" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)(IntroCrawl, {}),
			mode === "world" && /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_jsx_runtime.Fragment, { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(WorldCanvas, {}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)(WorldHud, {})] }),
			mode === "town" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)(TownView, {}),
			mode === "combat" && /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_jsx_runtime.Fragment, { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(HexCanvas, {}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)(CombatHud, {})] }),
			mode === "codex" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)(CodexView, {}),
			mode === "result" && /* @__PURE__ */ (0, import_jsx_runtime.jsx)(ResultView, {}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
				className: "pointer-events-auto absolute top-3 right-3 z-30 flex items-center gap-2",
				children: isPending ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", { className: "h-8 w-8 animate-pulse rounded-full bg-ash/10" }) : /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_jsx_runtime.Fragment, { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(SignedOut, { children: /* @__PURE__ */ (0, import_jsx_runtime.jsx)("a", {
					href: "/login",
					className: "rounded-md border border-line bg-fog px-3 py-1.5 text-xs tracking-wide text-dust hover:text-ash",
					children: "Sign in"
				}) }), /* @__PURE__ */ (0, import_jsx_runtime.jsx)(SignedIn, { children: /* @__PURE__ */ (0, import_jsx_runtime.jsx)(UserButton, {}) })] })
			})
		]
	});
}
function IntroCrawl() {
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("button", {
		type: "button",
		className: "absolute inset-0 flex flex-col items-center justify-center bg-ink px-6 text-left",
		onClick: () => useGame.getState().setMode("world"),
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("img", {
				src: "/art/title.jpg",
				alt: "",
				className: "absolute inset-0 h-full w-full object-cover opacity-25"
			}),
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", { className: "absolute inset-0 bg-linear-to-t from-ink via-ink/80 to-ink/50" }),
			/* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "relative max-w-lg",
				children: [
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "font-display text-xs tracking-[0.32em] text-dust uppercase",
						children: "Staffold · branding hall"
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("h2", {
						className: "mt-3 font-display text-4xl font-semibold",
						children: "You are No. 47."
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-4 text-sm leading-7 text-ash/85",
						children: "The Office put silver in your eyes and a sword too large for a human shoulder. Dovra has lit a beacon. If you walk slowly, the well will not be a well when you arrive."
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-4 text-sm leading-7 text-dust",
						children: "Raise the bar to use what they put in you. Lower it if you want to remain a person. The island does not care which you choose."
					}),
					/* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
						className: "mt-8 text-xs tracking-[0.22em] text-steel uppercase",
						children: "Tap to walk"
					})
				]
			})
		]
	});
}
function Home() {
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)(GameApp, {});
}
//#endregion
export { Home as component };
