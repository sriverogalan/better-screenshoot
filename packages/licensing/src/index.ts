export type LicenseTier = "community" | "pro" | "cloud" | "team";

export interface TierDefinition {
  id: LicenseTier;
  name: string;
  price: string;
  billing: "free" | "one-time" | "annual" | "monthly";
  features: string[];
}

export const TIERS: Record<LicenseTier, TierDefinition> = {
  community: {
    id: "community",
    name: "Community",
    price: "Free",
    billing: "free",
    features: [
      "Screen, window, and region capture",
      "Basic annotation",
      "Local history",
      "CLI and URL scheme",
      "Global shortcuts",
    ],
  },
  pro: {
    id: "pro",
    name: "Pro",
    price: "$24",
    billing: "one-time",
    features: [
      "Everything in Community",
      "Local OCR",
      "Scrolling capture",
      "GIF recording",
      "Pin to screen",
      "Custom themes",
    ],
  },
  cloud: {
    id: "cloud",
    name: "Cloud",
    price: "$6/mo",
    billing: "monthly",
    features: [
      "Everything in Pro",
      "Shareable links",
      "50 GB storage",
      "Custom domain",
      "Link expiration",
    ],
  },
  team: {
    id: "team",
    name: "Team",
    price: "$8/user/mo",
    billing: "monthly",
    features: [
      "Everything in Cloud",
      "SSO",
      "Team branding",
      "Admin panel",
      "Retention policies",
    ],
  },
};

export interface LicenseValidationResult {
  valid: boolean;
  tier: LicenseTier;
  expiresAt: string | null;
  message: string;
}

/** Placeholder — integrate Lemon Squeezy or Polar in production */
export async function validateLicenseKey(
  key: string,
  provider: "lemonsqueezy" | "polar" = "lemonsqueezy",
): Promise<LicenseValidationResult> {
  if (!key.trim()) {
    return {
      valid: true,
      tier: "community",
      expiresAt: null,
      message: "No key — Community mode",
    };
  }

  if (key.startsWith("BS-PRO-")) {
    return {
      valid: true,
      tier: "pro",
      expiresAt: null,
      message: `Valid Pro license (${provider})`,
    };
  }

  if (key.startsWith("BS-CLOUD-")) {
    return {
      valid: true,
      tier: "cloud",
      expiresAt: null,
      message: `Valid Cloud license (${provider})`,
    };
  }

  if (key.startsWith("BS-TEAM-")) {
    return {
      valid: true,
      tier: "team",
      expiresAt: null,
      message: `Valid Team license (${provider})`,
    };
  }

  return {
    valid: false,
    tier: "community",
    expiresAt: null,
    message: "Invalid license key",
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
