import {
  ClientCapabilities,
  NotificationType,
  WindowFeature,
} from 'vscode-languageserver';

export interface ProgressClientCapabilities {
  /**
   * Experimental client capabilities.
   */
  experimental: {
    /**
     * The client has support for reporting progress.
     */
    progress?: boolean;
  };
}

export interface ProgressParams {
  /**
   * A unique identifier to associate multiple progress notifications with the same progress.
   */
  id: string;

  /**
   * Mandatory title of the progress operation. Used to briefly inform about
   * the kind of operation being performed.
   * Examples: "Indexing" or "Linking dependencies".
   */
  title: string;

  /**
   * Optional, more detailed associated progress message. Contains
   * complementary information to the `title`.
   * Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
   * If unset, the previous progress message (if any) is still valid.
   */
  message?: string;

  /**
   * Optional progress percentage to display (value 100 is considered 100%).
   * If unset, the previous progress percentage (if any) is still valid.
   */
  percentage?: number;

  /**
   * Set to true on the final progress update.
   * No more progress notifications with the same ID should be sent.
   */
  done?: boolean;
}

export abstract class ProgressNotification {
  public static type = new NotificationType<ProgressParams, void>(
    'window/progress',
  );
}

export interface ProgressListener {
  progress(params: ProgressParams): void;
}

export const ProgressFeature: WindowFeature<ProgressListener> = Base => {
  return class extends Base {
    private progressCapabilities: ProgressClientCapabilities = {
      experimental: {},
    };

    public initialize(capabilities: ClientCapabilities) {
      capabilities.experimental = capabilities.experimental || {};
      this.progressCapabilities = capabilities as ProgressClientCapabilities;
    }

    public progress(params: ProgressParams): void {
      if (this.progressCapabilities.experimental.progress !== undefined) {
        this.connection.sendNotification(ProgressNotification.type, params);
      }
    }
  };
};
