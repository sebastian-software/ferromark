import { family } from "ferramenta-family/registry";

export const tool = family.find((member) => member.name === "ferromark");
if (!tool) throw new Error("Missing Ferromark registry entry");
