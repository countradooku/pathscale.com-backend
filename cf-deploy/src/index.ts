import { Container, getContainer } from "@cloudflare/containers";

export interface Env {
  PATHSCALE_CONTAINER: DurableObjectNamespace<PathscaleContainer>;
  // wrangler.toml [vars]
  HONEY_ID_ADDR: string;
  DB_PATH: string;
  R2_ACCOUNT_ID: string;
  R2_BUCKET_NAME: string;
  R2_PREFIX?: string;
  ADMIN_PUB_ID?: string;
  // Secrets — set via `wrangler secret put <NAME>` before first deploy
  HONEY_ID_APP_PUBLIC_ID: string;
  HONEY_ID_AUTH_API_KEY: string;
  TG_BOT_TOKEN?: string;
  // R2/Tigris FUSE mount credentials — set via `wrangler secret put`
  AWS_ACCESS_KEY_ID: string;
  AWS_SECRET_ACCESS_KEY: string;
}

export class PathscaleContainer extends Container {
  defaultPort = 8080;
  sleepAfter = "24h";
  enableInternet = true;
  
  constructor(ctx: Parameters<typeof Container>[0], env: Env) {
    super(ctx, env);
    // Container base class defines `envVars = {}` as a class field (own property),
    // which shadows prototype getters — must assign in constructor after super().
    this.envVars = {
      HONEY_ID_ADDR: env.HONEY_ID_ADDR,
      DB_PATH: env.DB_PATH,
      HONEY_ID_APP_PUBLIC_ID: env.HONEY_ID_APP_PUBLIC_ID,
      HONEY_ID_AUTH_API_KEY: env.HONEY_ID_AUTH_API_KEY,
      R2_ACCOUNT_ID: env.R2_ACCOUNT_ID,
      R2_BUCKET_NAME: env.R2_BUCKET_NAME,
      ...(env.R2_PREFIX ? { R2_PREFIX: env.R2_PREFIX } : {}),
      ...(env.ADMIN_PUB_ID ? { ADMIN_PUB_ID: env.ADMIN_PUB_ID } : {}),
      ...(env.TG_BOT_TOKEN ? { TG_BOT_TOKEN: env.TG_BOT_TOKEN } : {}),
      AWS_ACCESS_KEY_ID: env.AWS_ACCESS_KEY_ID,
      AWS_SECRET_ACCESS_KEY: env.AWS_SECRET_ACCESS_KEY,
    };
  }

  override async onStart(): Promise<void> {
    console.log("pathscale-be container started");
  }
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    return getContainer(env.PATHSCALE_CONTAINER, "primary").fetch(request);
  },
};
