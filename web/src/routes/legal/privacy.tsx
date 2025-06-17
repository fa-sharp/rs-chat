import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/legal/privacy")({
  component: PrivacyPolicy,
});

function PrivacyPolicy() {
  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8 max-w-4xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">Privacy Policy</h1>
          <p className="text-muted-foreground">Last updated: June 17, 2025</p>
        </div>

        <div className="prose prose-neutral dark:prose-invert max-w-none">
          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">1. Introduction</h2>
            <p>
              This Privacy Policy describes how RsChat ("we," "our," or "us")
              collects, uses, and protects your information when you use our
              chat application service. We are committed to protecting your
              privacy and being transparent about our data practices.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              2. Information We Collect
            </h2>

            <h3 className="text-xl font-medium mb-2">
              2.1 Authentication Information
            </h3>
            <p>When you log in using GitHub, we collect:</p>
            <ul className="list-disc pl-6 mb-4">
              <li>Your GitHub user ID</li>
              <li>Your GitHub username</li>
              <li>Your public profile information (name, avatar)</li>
            </ul>

            <h3 className="text-xl font-medium mb-2">2.2 API Keys</h3>
            <p>
              You voluntarily provide API keys for third-party AI services (such
              as Anthropic, OpenRouter). These keys are:
            </p>
            <ul className="list-disc pl-6 mb-4">
              <li>Encrypted before storage</li>
              <li>
                Used only to authenticate requests to AI providers on your
                behalf
              </li>
              <li>Never shared with other users or third parties</li>
            </ul>

            <h3 className="text-xl font-medium mb-2">
              2.3 Technical Information
            </h3>
            <p>We may collect basic technical information including:</p>
            <ul className="list-disc pl-6">
              <li>Browser type and version</li>
              <li>Operating system</li>
              <li>IP address (for security purposes)</li>
              <li>Access times and dates</li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              3. Information We Do NOT Collect
            </h2>
            <p>We want to be clear about what we don't collect:</p>
            <ul className="list-disc pl-6">
              <li>
                <strong>Analytics data:</strong> We do not use analytics
                services or tracking cookies
              </li>
              <li>
                <strong>Location data:</strong> We do not collect or track your
                location
              </li>
              <li>
                <strong>Behavioral data:</strong> We do not profile your usage
                patterns or preferences
              </li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              4. How We Use Your Information
            </h2>
            <p>We use the collected information solely to:</p>
            <ul className="list-disc pl-6">
              <li>Authenticate you and maintain your session</li>
              <li>Connect to AI providers using your API keys</li>
              <li>Provide the core chat functionality</li>
              <li>Store and manage your chat history</li>
              <li>Maintain the security and integrity of our service</li>
              <li>Comply with legal obligations if required</li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              5. Data Storage and Security
            </h2>

            <h3 className="text-xl font-medium mb-2">5.1 Encryption</h3>
            <p>
              All sensitive data, including API keys, is encrypted both in
              transit and at rest using industry-standard encryption methods.
            </p>

            <h3 className="text-xl font-medium mb-2">5.2 Data Retention</h3>
            <p>
              We retain your account information, chat history, and API keys
              only as long as your account is active. When you delete your
              account, all associated data is permanently removed.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">6. Data Sharing</h2>
            <p>
              We do not sell, trade, or share your personal information with
              third parties, except:
            </p>
            <ul className="list-disc pl-6">
              <li>
                <strong>AI Providers:</strong> Your API requests are sent to the
                AI providers you've configured (e.g., Anthropic, OpenRouter)
              </li>
              <li>
                <strong>GitHub:</strong> We use GitHub OAuth for authentication,
                which is governed by GitHub's privacy policy
              </li>
              <li>
                <strong>Legal Requirements:</strong> We may disclose information
                if required by law or to protect our rights
              </li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              7. Third-Party Services
            </h2>
            <p>
              When you use RsChat, you interact with third-party AI services.
              Please note:
            </p>
            <ul className="list-disc pl-6">
              <li>
                Each AI provider has their own privacy policy and data practices
              </li>
              <li>
                Your chat messages are sent directly to the AI providers you
                choose
              </li>
              <li>
                We recommend reviewing the privacy policies of AI providers you
                use
              </li>
              <li>
                We are not responsible for the privacy practices of third-party
                services
              </li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              8. Your Rights and Choices
            </h2>
            <p>You have the right to:</p>
            <ul className="list-disc pl-6">
              <li>
                <strong>Access:</strong> View the personal information we have
                about you
              </li>
              <li>
                <strong>Update:</strong> Modify your account information
              </li>
              <li>
                <strong>Delete:</strong> Remove your API keys or delete your
                account entirely
              </li>
              <li>
                <strong>Export:</strong> Request a copy of your account data
              </li>
              <li>
                <strong>Withdraw consent:</strong> Stop using the service at any
                time
              </li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              9. Children's Privacy
            </h2>
            <p>
              Our service is not intended for children under 13 years of age. We
              do not knowingly collect personal information from children under
              13. If you are a parent or guardian and believe your child has
              provided us with personal information, please contact us.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              10. International Users
            </h2>
            <p>
              RsChat may be accessed from around the world. By using our
              service, you consent to the transfer of your information to
              countries that may have different data protection laws than your
              country.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              11. Changes to This Privacy Policy
            </h2>
            <p>
              We may update this Privacy Policy from time to time. We will
              notify you of any changes by posting the new Privacy Policy on
              this page with an updated "Last updated" date. Changes are
              effective immediately upon posting.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">12. Contact Us</h2>
            <p>
              If you have any questions about this Privacy Policy or our data
              practices, please contact us through our GitHub repository or
              other available channels.
            </p>
          </section>
        </div>

        <div className="mt-12 pt-8 border-t border-border">
          <div className="flex items-center justify-between">
            <a href="/legal/terms" className="text-primary hover:underline">
              ← Terms of Service
            </a>
            <a href="/" className="text-primary hover:underline">
              Back to RsChat →
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
