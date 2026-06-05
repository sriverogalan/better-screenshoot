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
    price: "Gratis",
    billing: "free",
    features: [
      "Captura pantalla, ventana y región",
      "Anotación básica",
      "Historial local",
      "CLI y URL scheme",
      "Atajos globales",
    ],
  },
  pro: {
    id: "pro",
    name: "Pro",
    price: "$24",
    billing: "one-time",
    features: [
      "Todo Community",
      "OCR local",
      "Scrolling capture",
      "GIF recording",
      "Pin to screen",
      "Temas personalizados",
    ],
  },
  cloud: {
    id: "cloud",
    name: "Cloud",
    price: "$6/mes",
    billing: "monthly",
    features: [
      "Todo Pro",
      "Links compartibles",
      "50 GB almacenamiento",
      "Dominio propio",
      "Expiración de links",
    ],
  },
  team: {
    id: "team",
    name: "Team",
    price: "$8/usuario/mes",
    billing: "monthly",
    features: [
      "Todo Cloud",
      "SSO",
      "Branding de equipo",
      "Panel de administración",
      "Políticas de retención",
    ],
  },
};

export interface LicenseValidationResult {
  valid: boolean;
  tier: LicenseTier;
  expiresAt: string | null;
  message: string;
}

/** Placeholder — integrar Lemon Squeezy o Polar en producción */
export async function validateLicenseKey(
  key: string,
  provider: "lemonsqueezy" | "polar" = "lemonsqueezy",
): Promise<LicenseValidationResult> {
  if (!key.trim()) {
    return {
      valid: true,
      tier: "community",
      expiresAt: null,
      message: "Sin clave — modo Community",
    };
  }

  if (key.startsWith("BS-PRO-")) {
    return {
      valid: true,
      tier: "pro",
      expiresAt: null,
      message: `Licencia Pro válida (${provider})`,
    };
  }

  if (key.startsWith("BS-CLOUD-")) {
    return {
      valid: true,
      tier: "cloud",
      expiresAt: null,
      message: `Licencia Cloud válida (${provider})`,
    };
  }

  if (key.startsWith("BS-TEAM-")) {
    return {
      valid: true,
      tier: "team",
      expiresAt: null,
      message: `Licencia Team válida (${provider})`,
    };
  }

  return {
    valid: false,
    tier: "community",
    expiresAt: null,
    message: "Clave de licencia no válida",
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

/** Beta stub — reemplazar con API real de cloud */
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
