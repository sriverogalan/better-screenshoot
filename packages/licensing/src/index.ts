export type LicenseTier = "community" | "pro" | "cloud" | "team";

export interface TierDefinition {
  id: LicenseTier;
  price: string;
  billing: "free" | "one-time" | "annual" | "monthly";
}

export const TIERS: Record<LicenseTier, TierDefinition> = {
  community: {
    id: "community",
    price: "Free",
    billing: "free",
  },
  pro: {
    id: "pro",
    price: "$24",
    billing: "one-time",
  },
  cloud: {
    id: "cloud",
    price: "$6/mo",
    billing: "monthly",
  },
  team: {
    id: "team",
    price: "$8/user/mo",
    billing: "monthly",
  },
};

export interface LicenseValidationResult {
  valid: boolean;
  tier: LicenseTier;
  expiresAt: string | null;
  messageCode: string;
}

/** Placeholder — integrate Lemon Squeezy or Polar in production */
export async function validateLicenseKey(
  key: string,
  provider: "lemonsqueezy" | "polar" = "lemonsqueezy",
): Promise<LicenseValidationResult> {
  void provider;

  if (!key.trim()) {
    return {
      valid: true,
      tier: "community",
      expiresAt: null,
      messageCode: "noKey",
    };
  }

  if (key.startsWith("BS-PRO-")) {
    return {
      valid: true,
      tier: "pro",
      expiresAt: null,
      messageCode: "validPro",
    };
  }

  if (key.startsWith("BS-CLOUD-")) {
    return {
      valid: true,
      tier: "cloud",
      expiresAt: null,
      messageCode: "validCloud",
    };
  }

  if (key.startsWith("BS-TEAM-")) {
    return {
      valid: true,
      tier: "team",
      expiresAt: null,
      messageCode: "validTeam",
    };
  }

  return {
    valid: false,
    tier: "community",
    expiresAt: null,
    messageCode: "invalidKey",
  };
}

export interface CloudShareRequest {
  filePath: string;
  expiresInHours?: number;
}

export interface CloudShareResponse {
  url: string;
  expiresAt: string;
}

/** Beta stub — replace with real cloud API */
export async function uploadForShare(
  request: CloudShareRequest,
): Promise<CloudShareResponse> {
  const expiresAt = new Date(
    Date.now() + (request.expiresInHours ?? 168) * 60 * 60 * 1000,
  ).toISOString();

  return {
    url: `https://share.betterscreenshoot.app/beta/${encodeURIComponent(request.filePath.split("/").pop() ?? "capture.png")}`,
    expiresAt,
  };
}
