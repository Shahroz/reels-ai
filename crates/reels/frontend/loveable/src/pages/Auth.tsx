import { useState, useEffect } from 'react';
import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Video, Loader2 } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useToast } from '@/components/ui/use-toast';
import { z } from 'zod';

const emailSchema = z.string().email('Invalid email address');
const passwordSchema = z.string().min(6, 'Password must be at least 6 characters');
const nameSchema = z.string().min(2, 'Name must be at least 2 characters').max(100);

const Auth = () => {
  const { signUp, signIn, user } = useAuth();
  const navigate = useNavigate();
  const { toast } = useToast();
  const [isLoading, setIsLoading] = useState(false);
  
  const [signUpData, setSignUpData] = useState({
    email: '',
    password: '',
    fullName: '',
  });
  
  const [signInData, setSignInData] = useState({
    email: '',
    password: '',
  });

  useEffect(() => {
    if (user) {
      navigate('/dashboard');
    }
  }, [user, navigate]);

  const validateEmail = (email: string) => {
    try {
      emailSchema.parse(email);
      return { valid: true, error: null };
    } catch (error) {
      if (error instanceof z.ZodError) {
        return { valid: false, error: error.errors[0].message };
      }
      return { valid: false, error: 'Invalid email' };
    }
  };

  const validatePassword = (password: string) => {
    try {
      passwordSchema.parse(password);
      return { valid: true, error: null };
    } catch (error) {
      if (error instanceof z.ZodError) {
        return { valid: false, error: error.errors[0].message };
      }
      return { valid: false, error: 'Invalid password' };
    }
  };

  const handleSignUp = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const emailValidation = validateEmail(signUpData.email);
    if (!emailValidation.valid) {
      toast({ title: 'Invalid Email', description: emailValidation.error, variant: 'destructive' });
      return;
    }

    const passwordValidation = validatePassword(signUpData.password);
    if (!passwordValidation.valid) {
      toast({ title: 'Invalid Password', description: passwordValidation.error, variant: 'destructive' });
      return;
    }

    try {
      nameSchema.parse(signUpData.fullName);
    } catch (error) {
      if (error instanceof z.ZodError) {
        toast({ title: 'Invalid Name', description: error.errors[0].message, variant: 'destructive' });
        return;
      }
    }

    setIsLoading(true);
    const { error } = await signUp(signUpData.email, signUpData.password, signUpData.fullName);
    setIsLoading(false);

    if (error) {
      toast({
        title: 'Sign Up Failed',
        description: error.message,
        variant: 'destructive',
      });
    } else {
      toast({
        title: 'Account Created!',
        description: 'Welcome to VideoGen! You have 10 free credits.',
      });
      navigate('/dashboard');
    }
  };

  const handleSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const emailValidation = validateEmail(signInData.email);
    if (!emailValidation.valid) {
      toast({ title: 'Invalid Email', description: emailValidation.error, variant: 'destructive' });
      return;
    }

    const passwordValidation = validatePassword(signInData.password);
    if (!passwordValidation.valid) {
      toast({ title: 'Invalid Password', description: passwordValidation.error, variant: 'destructive' });
      return;
    }

    setIsLoading(true);
    const { error } = await signIn(signInData.email, signInData.password);
    setIsLoading(false);

    if (error) {
      toast({
        title: 'Sign In Failed',
        description: error.message,
        variant: 'destructive',
      });
    }
  };

  return (
    <div className="min-h-screen gradient-hero flex items-center justify-center p-6">
      <Card className="w-full max-w-md glass-card animate-scale-in">
        <CardHeader className="text-center space-y-4">
          <div className="w-16 h-16 rounded-xl gradient-accent flex items-center justify-center mx-auto">
            <Video className="w-10 h-10 text-white" />
          </div>
          <CardTitle className="text-3xl font-bold">Welcome to VideoGen</CardTitle>
          <CardDescription>Transform products into video pages with AI</CardDescription>
        </CardHeader>
        
        <CardContent>
          <Tabs defaultValue="signin" className="w-full">
            <TabsList className="grid w-full grid-cols-2 mb-8">
              <TabsTrigger value="signin">Sign In</TabsTrigger>
              <TabsTrigger value="signup">Sign Up</TabsTrigger>
            </TabsList>
            
            <TabsContent value="signin">
              <form onSubmit={handleSignIn} className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="signin-email">Email</Label>
                  <Input
                    id="signin-email"
                    type="email"
                    placeholder="you@example.com"
                    value={signInData.email}
                    onChange={(e) => setSignInData({ ...signInData, email: e.target.value })}
                    required
                    disabled={isLoading}
                  />
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="signin-password">Password</Label>
                  <Input
                    id="signin-password"
                    type="password"
                    placeholder="••••••••"
                    value={signInData.password}
                    onChange={(e) => setSignInData({ ...signInData, password: e.target.value })}
                    required
                    disabled={isLoading}
                  />
                </div>
                
                <Button type="submit" className="w-full gradient-accent text-white" disabled={isLoading}>
                  {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Sign In
                </Button>
              </form>
            </TabsContent>
            
            <TabsContent value="signup">
              <form onSubmit={handleSignUp} className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="signup-name">Full Name</Label>
                  <Input
                    id="signup-name"
                    type="text"
                    placeholder="John Doe"
                    value={signUpData.fullName}
                    onChange={(e) => setSignUpData({ ...signUpData, fullName: e.target.value })}
                    required
                    disabled={isLoading}
                  />
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="signup-email">Email</Label>
                  <Input
                    id="signup-email"
                    type="email"
                    placeholder="you@example.com"
                    value={signUpData.email}
                    onChange={(e) => setSignUpData({ ...signUpData, email: e.target.value })}
                    required
                    disabled={isLoading}
                  />
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="signup-password">Password</Label>
                  <Input
                    id="signup-password"
                    type="password"
                    placeholder="••••••••"
                    value={signUpData.password}
                    onChange={(e) => setSignUpData({ ...signUpData, password: e.target.value })}
                    required
                    disabled={isLoading}
                  />
                </div>
                
                <Button type="submit" className="w-full gradient-accent text-white" disabled={isLoading}>
                  {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Create Account
                </Button>
                
                <p className="text-xs text-center text-muted-foreground">
                  Start with 10 free credits • No credit card required
                </p>
              </form>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
};

export default Auth;