import type { LicenseTier } from "@better-screenshoot/shared-types";
import { TIERS, type TierDefinition } from "@better-screenshoot/licensing";
import type { Composer } from "vue-i18n";

export interface LocalizedTierDefinition extends TierDefinition {
  name: string;
  features: string[];
}

export function getLocalizedTier(
  tier: LicenseTier,
  t: Composer["t"],
  tm: Composer["tm"],
): LocalizedTierDefinition {
  const base = TIERS[tier];
  return {
    ...base,
    name: t(`licensing.tiers.${tier}.name`),
    features: tm(`licensing.tiers.${tier}.features`) as string[],
  };
}

export function getLocalizedTiers(
  t: Composer["t"],
  tm: Composer["tm"],
): Record<LicenseTier, LocalizedTierDefinition> {
  return {
    community: getLocalizedTier("community", t, tm),
    pro: getLocalizedTier("pro", t, tm),
    cloud: getLocalizedTier("cloud", t, tm),
    team: getLocalizedTier("team", t, tm),
  };
}

export function translateLicenseMessage(
  t: Composer["t"],
  code: string,
): string {
  const key = `licensing.messages.${code}`;
  const translated = t(key);
  return translated === key ? code : translated;
}
