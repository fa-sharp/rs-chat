import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/legal/terms")({
  component: TermsOfService,
});

function TermsOfService() {
  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8 max-w-4xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">Terms of Service</h1>
          <p className="text-muted-foreground">Last updated: June 17, 2025</p>
        </div>

        <div className="prose prose-neutral dark:prose-invert max-w-none">
          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              1. Acceptance of Terms
            </h2>
            <p>
              By accessing and using RsChat ("the Service"), you accept and
              agree to be bound by the terms and provision of this agreement. If
              you do not agree to abide by the above, please do not use this
              service.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              2. Description of Service
            </h2>
            <p>
              RsChat is a web application that allows users to interact with
              various AI language models through a chat interface. The Service
              acts as a client that connects to third-party AI providers using
              API keys that you provide.
            </p>
            <p>
              <strong>Important:</strong> You are responsible for obtaining and
              managing your own API keys from AI providers such as Anthropic,
              OpenRouter, and others. RsChat does not provide access to AI
              models directly.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              3. User Responsibilities
            </h2>
            <h3 className="text-xl font-medium mb-2">3.1 API Keys and Costs</h3>
            <ul className="list-disc pl-6 mb-4">
              <li>
                You are solely responsible for obtaining valid API keys from
                third-party AI providers
              </li>
              <li>
                You are responsible for all costs and charges incurred through
                your use of third-party AI services
              </li>
              <li>
                You must comply with the terms of service of each AI provider
                whose services you access
              </li>
              <li>
                You are responsible for monitoring your API usage and associated
                costs
              </li>
            </ul>

            <h3 className="text-xl font-medium mb-2">3.2 Account Security</h3>
            <ul className="list-disc pl-6 mb-4">
              <li>
                You are responsible for maintaining the security of your GitHub
                account used for authentication
              </li>
              <li>
                You must keep your API keys secure and not share them with
                unauthorized parties
              </li>
              <li>
                You must notify us immediately of any unauthorized use of your
                account
              </li>
            </ul>

            <h3 className="text-xl font-medium mb-2">3.3 Acceptable Use</h3>
            <ul className="list-disc pl-6">
              <li>
                You will not use the Service for any illegal or unauthorized
                purpose
              </li>
              <li>
                You will not violate any laws in your jurisdiction when using
                the Service
              </li>
              <li>
                You will not attempt to interfere with or disrupt the Service
              </li>
              <li>
                You will comply with all applicable third-party AI provider
                terms and policies
              </li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">4. Privacy and Data</h2>
            <p>
              Your privacy is important to us. Please review our Privacy Policy,
              which also governs your use of the Service, to understand our
              practices.
            </p>
            <ul className="list-disc pl-6">
              <li>
                We do not share your data with third parties beyond what is
                necessary to provide the Service
              </li>
              <li>Your API keys are encrypted and stored securely</li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              5. Third-Party Services
            </h2>
            <p>
              The Service integrates with third-party AI providers and other
              services. We are not responsible for:
            </p>
            <ul className="list-disc pl-6">
              <li>
                The availability, accuracy, or reliability of third-party
                services
              </li>
              <li>
                The terms, conditions, or policies of third-party providers
              </li>
              <li>
                Any costs, charges, or damages arising from your use of
                third-party services
              </li>
              <li>The content or responses generated by AI models</li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">6. Disclaimers</h2>
            <p>
              The Service is provided "as is" without any warranties, expressed
              or implied. We do not warrant that:
            </p>
            <ul className="list-disc pl-6">
              <li>The Service will be uninterrupted or error-free</li>
              <li>
                The results obtained from the Service will be accurate or
                reliable
              </li>
              <li>Any defects in the Service will be corrected</li>
              <li>
                Third-party AI services will be available or function as
                expected
              </li>
            </ul>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              7. Limitation of Liability
            </h2>
            <p>
              In no event shall RsChat be liable for any indirect, incidental,
              special, consequential, or punitive damages, including without
              limitation, loss of profits, data, use, goodwill, or other
              intangible losses, resulting from your use of the Service.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">8. Indemnification</h2>
            <p>
              You agree to indemnify and hold harmless RsChat and its affiliates
              from any claims, damages, or expenses arising from your use of
              third-party AI services, violation of these terms, or infringement
              of any third-party rights.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">9. Termination</h2>
            <p>
              We may terminate or suspend your access to the Service at any
              time, without prior notice or liability, for any reason
              whatsoever, including without limitation if you breach the Terms.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              10. Changes to Terms
            </h2>
            <p>
              We reserve the right to modify these terms at any time. We will
              notify users of any material changes by posting the new Terms of
              Service on this page with an updated "Last updated" date.
            </p>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4">
              11. Contact Information
            </h2>
            <p>
              If you have any questions about these Terms of Service, please
              contact us through our GitHub repository or other available
              channels.
            </p>
          </section>
        </div>

        <div className="mt-12 pt-8 border-t border-border">
          <div className="flex items-center justify-between">
            <a href="/" className="text-primary hover:underline">
              ← Back to RsChat
            </a>
            <a href="/legal/privacy" className="text-primary hover:underline">
              Privacy Policy →
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
