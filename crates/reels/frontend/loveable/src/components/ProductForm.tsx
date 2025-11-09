import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useToast } from '@/components/ui/use-toast';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from '@/hooks/useAuth';
import { useReelGeneration } from '@/hooks/useReelGeneration';
import { Loader2, X, CheckCircle2, AlertCircle, Info, XCircle } from 'lucide-react';

interface ProductFormProps {
  onClose: () => void;
}

const ProductForm = ({ onClose }: ProductFormProps) => {
  const { user } = useAuth();
  const { toast } = useToast();
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    prompt: 'Create a 15-second promotional reel about sustainable energy solutions',
    productUrl: '',
    duration: 15,
  });

  const {
    isGenerating,
    statusMessages,
    logEntries,
    videoUrl,
    videoInfo,
    generateReel,
    cancelGeneration,
  } = useReelGeneration();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!user) return;

    // Check credits before starting
    const { data: creditData } = await supabase
      .from('credits')
      .select('balance, total_spent')
      .eq('user_id', user.id)
      .single();

    if (!creditData || creditData.balance < 1) {
      toast({
        title: 'Insufficient Credits',
        description: 'You need at least 1 credit to generate a video page.',
        variant: 'destructive',
      });
      return;
    }

    // Generate reel using the hook
    generateReel(
      formData.prompt || formData.description || `Create a promotional reel about ${formData.title}`,
      formData.productUrl || null,
      formData.duration
    );
  };

  const handleVideoComplete = async () => {
    if (!user || !videoUrl || !videoInfo) return;

    try {
      // Check credits
      const { data: creditData } = await supabase
        .from('credits')
        .select('balance, total_spent')
        .eq('user_id', user.id)
        .single();

      if (!creditData || creditData.balance < 1) {
        toast({
          title: 'Insufficient Credits',
          description: 'You need at least 1 credit to save the video page.',
          variant: 'destructive',
        });
        return;
      }

      // Create product
      const { data: product, error: productError } = await supabase
        .from('products')
        .insert({
          user_id: user.id,
          title: formData.title || 'Generated Reel',
          description: formData.description || formData.prompt,
          category: '',
          price: 0,
          images: [],
          status: 'completed',
        })
        .select()
        .single();

      if (productError) throw productError;

      // Create video page
      const { error: videoError } = await supabase
        .from('video_pages')
        .insert({
          user_id: user.id,
          product_id: product.id,
          status: 'completed',
          credits_used: 1,
          share_url: `https://videogen.app/v/${product.id}`,
          video_url: videoUrl,
        });

      if (videoError) throw videoError;

      // Deduct credits
      await supabase
        .from('credits')
        .update({
          balance: creditData.balance - 1,
          total_spent: (creditData.total_spent || 0) + 1,
        })
        .eq('user_id', user.id);

      toast({
        title: 'Success!',
        description: 'Your video page has been generated and saved.',
      });

      onClose();
    } catch (error) {
      console.error('Error saving video page:', error);
      toast({
        title: 'Error',
        description: 'Failed to save video page. Please try again.',
        variant: 'destructive',
      });
    }
  };

  const getStatusIcon = (type: string) => {
    switch (type) {
      case 'success':
        return <CheckCircle2 className="w-4 h-4 text-green-500" />;
      case 'error':
        return <XCircle className="w-4 h-4 text-red-500" />;
      case 'warning':
        return <AlertCircle className="w-4 h-4 text-yellow-500" />;
      default:
        return <Info className="w-4 h-4 text-blue-500" />;
    }
  };

  return (
    <div className="space-y-6">
      <Card className="glass-card animate-scale-in">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-2xl">Create Video Page</CardTitle>
              <CardDescription>Enter product details to generate your video page</CardDescription>
            </div>
            <Button variant="ghost" size="icon" onClick={onClose} disabled={isGenerating}>
              <X className="w-5 h-5" />
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="title">Product Title *</Label>
              <Input
                id="title"
                placeholder="Amazing Product Name"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                required
                disabled={isGenerating}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                placeholder="Describe your product..."
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={4}
                disabled={isGenerating}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="prompt">Reel Prompt *</Label>
              <Textarea
                id="prompt"
                placeholder="Describe the reel you want to generate... e.g., 'Create a 15-second promotional reel about a new fitness app with energetic music and dynamic visuals'"
                value={formData.prompt}
                onChange={(e) => setFormData({ ...formData, prompt: e.target.value })}
                required
                rows={4}
                disabled={isGenerating}
              />
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <Label htmlFor="productUrl">Product/Service URL (Optional)</Label>
                <Input
                  id="productUrl"
                  type="url"
                  placeholder="https://example.com/product"
                  value={formData.productUrl}
                  onChange={(e) => setFormData({ ...formData, productUrl: e.target.value })}
                  disabled={isGenerating}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="duration">Duration (seconds)</Label>
                <Input
                  id="duration"
                  type="number"
                  min={5}
                  max={60}
                  value={formData.duration}
                  onChange={(e) => setFormData({ ...formData, duration: parseInt(e.target.value) || 15 })}
                  disabled={isGenerating}
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-4 border-t">
              <p className="text-sm text-muted-foreground">This will use 1 credit</p>
              <div className="flex gap-2">
                {isGenerating && (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={cancelGeneration}
                  >
                    Cancel
                  </Button>
                )}
                <Button
                  type="submit"
                  className="gradient-accent text-white"
                  disabled={isGenerating || !formData.prompt.trim()}
                >
                  {isGenerating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  {isGenerating ? 'Generating...' : 'Generate Video Page'}
                </Button>
              </div>
            </div>
          </form>
        </CardContent>
      </Card>

      {/* Status Messages */}
      {statusMessages.length > 0 && (
        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-lg">Status</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {statusMessages.slice(-5).map((msg, index) => (
                <Alert key={index} variant={msg.type === 'error' ? 'destructive' : 'default'}>
                  <div className="flex items-center gap-2">
                    {getStatusIcon(msg.type)}
                    <AlertDescription>{msg.message}</AlertDescription>
                  </div>
                </Alert>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Generated Video */}
      {videoUrl && (
        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-lg">Generated Reel</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="aspect-video bg-black rounded-lg overflow-hidden">
              <video src={videoUrl} controls preload="auto" className="w-full h-full">
                Your browser does not support the video tag.
              </video>
            </div>
            {videoInfo && (
              <div className="space-y-2 text-sm">
                {videoInfo.reelId && (
                  <p><strong>Reel ID:</strong> {videoInfo.reelId}</p>
                )}
                {videoInfo.duration && (
                  <p><strong>Duration:</strong> {videoInfo.duration} seconds</p>
                )}
              </div>
            )}
            <Button
              onClick={handleVideoComplete}
              className="gradient-accent text-white w-full"
            >
              Save Video Page
            </Button>
          </CardContent>
        </Card>
      )}

      {/* Log Entries */}
      {logEntries.length > 0 && (
        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-lg">Generation Log</CardTitle>
          </CardHeader>
          <CardContent>
            <ScrollArea className="h-64">
              <div className="space-y-2">
                {logEntries.map((entry, index) => (
                  <div
                    key={index}
                    className={`text-sm p-2 rounded ${
                      entry.type === 'error' ? 'bg-red-500/10 text-red-500' :
                      entry.type === 'success' ? 'bg-green-500/10 text-green-500' :
                      entry.type === 'warning' ? 'bg-yellow-500/10 text-yellow-500' :
                      'bg-blue-500/10 text-blue-500'
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      {getStatusIcon(entry.type)}
                      <span className="text-xs text-muted-foreground">
                        {entry.timestamp.toLocaleTimeString()}
                      </span>
                    </div>
                    <p className="mt-1">{entry.message}</p>
                  </div>
                ))}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default ProductForm;